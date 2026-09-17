package app.indelible.core.offline

import app.indelible.share.isNetworkException
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock

private const val MAX_READING_EVENT_BATCH = 200

class OutboxWorker(
    private val store: OfflineStore,
    private val scope: suspend () -> String?,
    private val sender: OutboxSender,
    private val clock: () -> Long,
    private val isNetwork: (Throwable) -> Boolean = ::isNetworkException,
) {
    private val drainLock = Mutex()
    private val triggerLock = Mutex()
    private var rerunRequested = false
    private val authPausedState = MutableStateFlow(false)
    val authPaused: StateFlow<Boolean> = authPausedState.asStateFlow()

    fun resumeAuth() {
        authPausedState.value = false
    }

    suspend fun nextRetryAt(): Long? {
        val currentScope = scope() ?: return null
        return store.earliestRetryAt(currentScope, clock())
    }

    suspend fun drain() {
        if (!acquireOrRequestRerun()) return
        try {
            do {
                drainLocked()
            } while (consumeRerunRequest())
        } finally {
            drainLock.unlock()
        }
    }

    /**
     * A trigger that loses the lock must not be dropped: [runDrain] snapshots its work once, so
     * anything enqueued after that snapshot only ships if the winner runs again. The flag is
     * never cleared on acquisition, so a request landing in the gap between the winner's last
     * check and its unlock costs one redundant pass rather than a lost row.
     */
    private suspend fun acquireOrRequestRerun(): Boolean =
        triggerLock.withLock {
            if (drainLock.tryLock()) {
                true
            } else {
                rerunRequested = true
                false
            }
        }

    private suspend fun consumeRerunRequest(): Boolean =
        triggerLock.withLock {
            rerunRequested.also { rerunRequested = false }
        }

    private suspend fun drainLocked() {
        val currentScope = scope() ?: return
        // scopesWithState() only lists scopes with a client_state row; the purge sweep needs
        // every scope that ever had local data to show up there, not only ones that enqueued.
        store.clientIdentity(currentScope)
        if (!authPausedState.value) {
            runDrain(currentScope)
        }
    }

    private suspend fun runDrain(currentScope: String) {
        val now = clock()
        val rows = store.drainable(currentScope, now)
        val shielded = mutableSetOf<String>()
        var index = 0
        while (index < rows.size) {
            val row = rows[index]
            if (row.entityId in shielded) {
                index++
                continue
            }
            val batch = nextBatch(rows, index, shielded)
            index += batch.size

            val outcome = sender.send(currentScope, batch)
            val classification = classify(outcome, batch.first().attempts, now, isNetwork)
            when (classification) {
                is Classification.Retryable -> {
                    applyRetryable(currentScope, batch, classification, outcome, now)
                    return
                }
                Classification.AuthBlocked -> {
                    authPausedState.value = true
                    return
                }
                is Classification.Terminal -> applyTerminal(currentScope, batch, classification, shielded)
                Classification.Done -> applyDone(currentScope, batch, now)
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
        now: Long,
    ) {
        val error = describe(outcome)
        for (row in batch) {
            store.markAttempt(scope, row.id, now, classification.nextAttemptAt, error)
        }
    }

    private suspend fun applyTerminal(
        scope: String,
        batch: List<OutboxRow>,
        classification: Classification.Terminal,
        shielded: MutableSet<String>,
    ) {
        for (row in batch) {
            store.markFailed(scope, row.id, classification.error)
            if (row.kind == OutboxKind.HIGHLIGHT_CREATE) {
                store.blockDependants(scope, row.entityId)
            }
            shielded += row.entityId
        }
    }

    private suspend fun applyDone(
        scope: String,
        batch: List<OutboxRow>,
        now: Long,
    ) {
        for (row in batch) {
            store.remove(scope, row.id)
        }
        // Every row in a batch shares documentId by construction (batching only folds rows for
        // the same document), so one sync stamp per batch is enough.
        store.markDocumentSynced(scope, batch.first().documentId, now)
    }

    private fun describe(outcome: SendOutcome): String =
        when (outcome) {
            is SendOutcome.Http -> "HTTP ${outcome.status}: ${outcome.message}"
            is SendOutcome.Transport -> outcome.cause.message ?: outcome.cause.toString()
            SendOutcome.Success, SendOutcome.ReplaySuccess -> "success"
        }
}
