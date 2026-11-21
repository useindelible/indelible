package app.indelible.core.offline

private const val MAX_READING_EVENT_BATCH = 200

internal sealed interface PassResult {
    data object Completed : PassResult

    data class RetryableStop(
        val nextAttemptAt: Long,
    ) : PassResult

    data object AuthPaused : PassResult

    data object Stale : PassResult

    data object NoSession : PassResult

    data object Failed : PassResult
}

/** What one walk has learnt so far about rows it must pass over. */
private class Walk {
    val shielded = mutableSetOf<String>()
    val skippedKinds = mutableSetOf<OutboxKind>()
    var heldUntil: Long? = null
}

internal class PassOutcome(
    val result: PassResult,
    val waitingDeadline: Long?,
    /** A due row was passed over because an earlier row of its entity failed in this walk. */
    val skippedDue: Boolean = false,
)

/**
 * One walk over the eligible rows of a session. Owns no timers and no readiness; the worker
 * does. The session is re-checked before every send so a transition that withdrew it ends the
 * pass at the next batch instead of shipping rows under credentials that no longer belong to it.
 */
internal class OutboxPass(
    private val store: OfflineStore,
    private val registry: SessionRegistry,
    private val sender: OutboxSender,
    private val clock: () -> Long,
    private val isNetwork: (Throwable) -> Boolean,
) {
    suspend fun run(session: Session): PassOutcome {
        val scope = session.scope
        val clientId = store.clientIdentity(scope).clientId
        val plan = eligibleRows(store.pendingOrdered(scope), clock())
        val rows = plan.admitted
        val walk = Walk()
        var skippedDue = false
        var stop: PassResult? = null
        var index = 0
        while (index < rows.size && stop == null) {
            val row = rows[index]
            if (row.entityId in walk.shielded || row.kind in walk.skippedKinds) {
                if (row.entityId in walk.shielded) skippedDue = true
                index++
                continue
            }
            val batch = nextBatch(rows, index, walk.shielded)
            index += batch.size
            stop =
                if (registry.current.value.session !== session) {
                    PassResult.Stale
                } else {
                    sendAndApply(session, clientId, batch, walk)
                }
        }
        stop = stop ?: walk.heldUntil?.let { PassResult.RetryableStop(it) }
        return when (stop) {
            null -> PassOutcome(PassResult.Completed, plan.earliestWaitingDeadline, skippedDue)
            is PassResult.RetryableStop -> PassOutcome(stop, plan.earliestWaitingDeadline)
            else -> PassOutcome(stop, null)
        }
    }

    /** Returns how many rows an old server refused were given another try. */
    suspend fun reviveServerTooOld(scope: String): Int =
        store
            .rowsByState(scope, OutboxState.FAILED)
            .filter { it.failure?.kind == SyncFailure.Kind.SERVER_TOO_OLD }
            .count { store.retryRow(scope, it.id) }

    /** Returns the result that ends the pass, or null when the next batch may follow. */
    private suspend fun sendAndApply(
        session: Session,
        clientId: String,
        batch: List<OutboxRow>,
        walk: Walk,
    ): PassResult? {
        val scope = session.scope
        val rows = currentRows(scope, batch) ?: return null
        val head = rows.first()
        val outcome = sender.send(session, clientId, rows)
        val outcomeAt = clock()
        return when (val classification = classify(head.kind, outcome, head.attempts, outcomeAt, isNetwork)) {
            is Classification.Retryable -> {
                applyRetryable(scope, rows, classification, outcome, outcomeAt)
                // Only the capabilities route went unanswered, so the rest of the queue still goes out in this walk.
                if (outcome == SendOutcome.NoAnswer) {
                    walk.skippedKinds += head.kind
                    walk.heldUntil = minOf(walk.heldUntil ?: classification.nextAttemptAt, classification.nextAttemptAt)
                    null
                } else {
                    PassResult.RetryableStop(classification.nextAttemptAt)
                }
            }
            Classification.AuthBlocked -> PassResult.AuthPaused
            Classification.Stale -> PassResult.Stale
            is Classification.Terminal -> {
                applyTerminal(scope, rows, classification, walk.shielded)
                // A missing route holds the whole kind; a missing capability holds only the rows that need it.
                val routeMissing = outcome != SendOutcome.Unsupported
                val tooOld = classification.failure.kind == SyncFailure.Kind.SERVER_TOO_OLD
                if (tooOld && routeMissing) walk.skippedKinds += head.kind
                null
            }
            Classification.Done -> {
                applyDone(scope, rows, outcome, outcomeAt)
                null
            }
        }
    }

    // An earlier note in this walk rebases the later ones in storage, so a note is sent as it is now, not as
    // it was read at the start; a row that is gone meanwhile is simply skipped.
    private suspend fun currentRows(
        scope: String,
        batch: List<OutboxRow>,
    ): List<OutboxRow>? {
        val head = batch.first()
        if (head.kind != OutboxKind.DOCUMENT_NOTE) return batch
        return store.pendingOrdered(scope).firstOrNull { it.id == head.id }?.let(::listOf)
    }

    private fun nextBatch(
        rows: List<OutboxRow>,
        start: Int,
        shielded: Set<String>,
    ): List<OutboxRow> {
        val head = rows[start]
        if (head.kind != OutboxKind.READING_EVENT) return listOf(head)

        val batch = mutableListOf(head)
        var j = start + 1
        while (j < rows.size && canAppendToBatch(head, rows[j], batch.size, shielded)) {
            batch += rows[j]
            j++
        }
        return batch
    }

    private fun canAppendToBatch(
        head: OutboxRow,
        candidate: OutboxRow,
        currentBatchSize: Int,
        shielded: Set<String>,
    ): Boolean {
        val withinBatchLimit = currentBatchSize < MAX_READING_EVENT_BATCH
        val sameDocument = candidate.kind == OutboxKind.READING_EVENT && candidate.documentId == head.documentId
        return withinBatchLimit && sameDocument && candidate.entityId !in shielded
    }

    private suspend fun applyRetryable(
        scope: String,
        batch: List<OutboxRow>,
        classification: Classification.Retryable,
        outcome: SendOutcome,
        outcomeAt: Long,
    ) {
        val error = describe(outcome)
        for (row in batch) {
            store.markAttempt(scope, row.id, outcomeAt, classification.nextAttemptAt, error)
        }
    }

    private suspend fun applyTerminal(
        scope: String,
        batch: List<OutboxRow>,
        classification: Classification.Terminal,
        shielded: MutableSet<String>,
    ) {
        for (row in batch) {
            if (row.kind == OutboxKind.HIGHLIGHT_CREATE) {
                store.failCreateAndBlockDependants(scope, row.id, row.entityId, classification.failure)
            } else {
                store.markFailed(scope, row.id, classification.failure)
            }
            shielded += row.entityId
        }
    }

    private suspend fun applyDone(
        scope: String,
        batch: List<OutboxRow>,
        outcome: SendOutcome,
        outcomeAt: Long,
    ) {
        val noteUpdatedAt = (outcome as? SendOutcome.Success)?.noteUpdatedAtEpochMs
        for (row in batch) {
            store.remove(scope, row.id, noteUpdatedAt)
        }
        // Every row in a batch shares documentId by construction (batching only folds rows for
        // the same document), so one sync stamp per batch is enough.
        store.markDocumentSynced(scope, batch.first().documentId, outcomeAt)
    }

    private fun describe(outcome: SendOutcome): String =
        when (outcome) {
            is SendOutcome.Http ->
                if (outcome.unpreparedContent) STILL_PREPARING_ERROR else "HTTP ${outcome.status}: ${outcome.message}"
            is SendOutcome.Transport -> outcome.cause.message ?: outcome.cause.toString()
            is SendOutcome.Success, SendOutcome.ReplaySuccess -> "success"
            is SendOutcome.NoteConflict -> "note changed on another device"
            SendOutcome.Unsupported -> "the server needs updating"
            SendOutcome.NoAnswer -> "the server could not be asked what it supports"
            SendOutcome.Stale -> "stale session"
        }
}
