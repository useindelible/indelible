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
)

/** One walk over the eligible rows of a scope. Owns no timers and no readiness; the worker does. */
internal class OutboxPass(
    private val store: OfflineStore,
    private val sender: OutboxSender,
    private val clock: () -> Long,
    private val isNetwork: (Throwable) -> Boolean,
) {
    suspend fun run(scope: String): PassOutcome {
        val clientId = store.clientIdentity(scope).clientId
        val plan = eligibleRows(store.pendingOrdered(scope), clock())
        val rows = plan.admitted
        val shielded = mutableSetOf<String>()
        var stop: PassResult? = null
        var index = 0
        while (index < rows.size && stop == null) {
            val row = rows[index]
            if (row.entityId in shielded) {
                index++
                continue
            }
            val batch = nextBatch(rows, index, shielded)
            index += batch.size

            val outcome = sender.send(scope, clientId, batch)
            val outcomeAt = clock()
            when (val classification = classify(outcome, batch.first().attempts, outcomeAt, isNetwork)) {
                is Classification.Retryable -> {
                    applyRetryable(scope, batch, classification, outcome, outcomeAt)
                    stop = PassResult.RetryableStop(classification.nextAttemptAt)
                }
                Classification.AuthBlocked -> stop = PassResult.AuthPaused
                is Classification.Terminal -> applyTerminal(scope, batch, classification, shielded)
                Classification.Done -> applyDone(scope, batch, outcomeAt)
            }
        }
        return when (stop) {
            null -> PassOutcome(PassResult.Completed, plan.earliestWaitingDeadline)
            PassResult.AuthPaused -> PassOutcome(stop, null)
            else -> PassOutcome(stop, plan.earliestWaitingDeadline)
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
        }
}
