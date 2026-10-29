package app.indelible.reader.viewmodel

import app.indelible.core.offline.ReadingPosition
import app.indelible.reader.repository.ReadingEventWriter
import kotlinx.coroutines.CompletableDeferred

/** Records what a reader would have queued; [failWith] makes every call fail instead. */
class RecordingReadingEventWriter(
    private val failWith: Throwable? = null,
) : ReadingEventWriter {
    val openedDocuments = mutableListOf<String>()

    /** When set, recordOpened waits for it, as a slow queue write would. */
    var openedGate: CompletableDeferred<Unit>? = null
    val progressPercents = mutableListOf<Float>()
    val positions = mutableListOf<ReadingPosition?>()

    override suspend fun recordOpened(
        documentId: String,
        sessionId: String,
    ) {
        failWith?.let { throw it }
        openedGate?.await()
        openedDocuments += documentId
    }

    override suspend fun recordProgress(
        documentId: String,
        percent: Float,
        sessionId: String,
        position: ReadingPosition?,
    ) {
        failWith?.let { throw it }
        progressPercents += percent
        positions += position
    }
}
