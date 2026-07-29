package app.indelible.reader.viewmodel

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlinx.datetime.Clock

internal class ReaderProgressSync(
    private val scope: CoroutineScope,
    private val updateProgress: suspend (Float) -> Unit,
) {
    private var syncJob: Job? = null
    private var pendingProgress: Float? = null
    private var syncInFlight = false
    private var flushAfterInFlight = false
    private var lastSyncAtMillis = 0L

    fun schedule(percent: Float) {
        pendingProgress = percent
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

    private fun flushPending() {
        if (syncInFlight) {
            flushAfterInFlight = true
            return
        }
        val percent = pendingProgress ?: return
        pendingProgress = null
        syncInFlight = true
        lastSyncAtMillis = Clock.System.now().toEpochMilliseconds()
        scope.launch {
            updateProgress(percent)
            syncInFlight = false
            if (flushAfterInFlight || pendingProgress != null) {
                flushAfterInFlight = false
                flushPending()
            }
        }
    }

    private companion object {
        const val IDLE_SYNC_MS = 800L
        const val ACTIVE_SYNC_MS = 5000L
    }
}
