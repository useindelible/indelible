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
        val shielded = mutableSetOf<String>()
        var skippedDue = false
        var stop: PassResult? = null
        var index = 0
        while (index < rows.size && stop == null) {
            val row = rows[index]
            if (row.entityId in shielded) {
                skippedDue = true
                index++
                continue
            }
            val batch = nextBatch(rows, index, shielded)
            index += batch.size
            stop =
                if (registry.current.value.session !== session) {
                    PassResult.Stale
                } else {
                    sendAndApply(session, clientId, batch, shielded)
                }
        }
        return when (stop) {
            null -> PassOutcome(PassResult.Completed, plan.earliestWaitingDeadline, skippedDue)
            is PassResult.RetryableStop -> PassOutcome(stop, plan.earliestWaitingDeadline)
            else -> PassOutcome(stop, null)
        }
    }

    /** Returns the result that ends the pass, or null when the next batch may follow. */
    private suspend fun sendAndApply(
        session: Session,
        clientId: String,
        batch: List<OutboxRow>,
        shielded: MutableSet<String>,
    ): PassResult? {
        val scope = session.scope
        val outcome = sender.send(session, clientId, batch)
        val outcomeAt = clock()
        return when (val classification = classify(outcome, batch.first().attempts, outcomeAt, isNetwork)) {
            is Classification.Retryable -> {
                applyRetryable(scope, batch, classification, outcome, outcomeAt)
                PassResult.RetryableStop(classification.nextAttemptAt)
            }
            Classification.AuthBlocked -> PassResult.AuthPaused
            Classification.Stale -> PassResult.Stale
            is Classification.Terminal -> {
                applyTerminal(scope, batch, classification, shielded)
                null
            }
            Classification.Done -> {
                applyDone(scope, batch, outcomeAt)
                null
            }
        }
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
                store.failCreateAndBlockDependants(scope, row.id, row.entityId, classification.error)
            } else {
                store.markFailed(scope, row.id, classification.error)
            }
            shielded += row.entityId
        }
    }

    private suspend fun applyDone(
        scope: String,
        batch: List<OutboxRow>,
        outcomeAt: Long,
    ) {
        for (row in batch) {
            store.remove(scope, row.id)
        }
        // Every row in a batch shares documentId by construction (batching only folds rows for
        // the same document), so one sync stamp per batch is enough.
        store.markDocumentSynced(scope, batch.first().documentId, outcomeAt)
    }

    private fun describe(outcome: SendOutcome): String =
        when (outcome) {
            is SendOutcome.Http -> "HTTP ${outcome.status}: ${outcome.message}"
            is SendOutcome.Transport -> outcome.cause.message ?: outcome.cause.toString()
            SendOutcome.Success, SendOutcome.ReplaySuccess -> "success"
            SendOutcome.Stale -> "stale session"
        }
}
