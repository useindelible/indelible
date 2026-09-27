package app.indelible.reader.viewmodel

import app.indelible.core.offline.ChapterPosition
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.CoroutineStart
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlinx.datetime.Clock

internal class ReaderProgressSync(
    private val scope: CoroutineScope,
    private val updateProgress: suspend (Float, ChapterPosition?) -> Unit,
) {
    private class Pending(
        val percent: Float,
        val position: ChapterPosition?,
    )

    private var syncJob: Job? = null
    private var inFlight: Job? = null
    private var pendingProgress: Pending? = null
    private var syncInFlight = false
    private var flushAfterInFlight = false
    private var lastSyncAtMillis = 0L

    fun schedule(
        percent: Float,
        position: ChapterPosition? = null,
    ) {
        pendingProgress = Pending(percent, position)
        val now = Clock.System.now().toEpochMilliseconds()
        if (now - lastSyncAtMillis >= ACTIVE_SYNC_MS) {
            syncJob?.cancel()
            syncJob = null
            flushPending()
            return
        }

        syncJob?.cancel()
        syncJob =
            scope.launch {
                delay(IDLE_SYNC_MS)
                syncJob = null
                flushPending()
            }
    }

    fun flush() {
        syncJob?.cancel()
        syncJob = null
        flushPending()
    }

    /** Flushes the pending progress and returns once every write it started has been enqueued. */
    suspend fun flushNow() {
        flush()
        while (true) {
            val job = inFlight ?: return
            job.join()
            if (inFlight === job) return
        }
    }

    private fun flushPending() {
        if (syncInFlight) {
            flushAfterInFlight = true
            return
        }
        val pending = pendingProgress ?: return
        pendingProgress = null
        syncInFlight = true
        lastSyncAtMillis = Clock.System.now().toEpochMilliseconds()
        val job =
            scope.launch(start = CoroutineStart.LAZY) {
                // A callback that fails must not strand the flag or the sample queued behind it.
                try {
                    updateProgress(pending.percent, pending.position)
                } finally {
                    syncInFlight = false
                    if (flushAfterInFlight || pendingProgress != null) {
                        flushAfterInFlight = false
                        flushPending()
                    }
                }
            }
        inFlight = job
        job.start()
    }

    private companion object {
        const val IDLE_SYNC_MS = 800L
        const val ACTIVE_SYNC_MS = 5000L
    }
}
