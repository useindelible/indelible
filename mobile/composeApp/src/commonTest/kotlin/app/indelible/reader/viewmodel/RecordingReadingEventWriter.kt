package app.indelible.reader.viewmodel

import app.indelible.reader.repository.ReadingEventWriter

/** Records what a reader would have queued; [failWith] makes every call fail instead. */
class RecordingReadingEventWriter(
    private val failWith: Throwable? = null,
) : ReadingEventWriter {
    val openedDocuments = mutableListOf<String>()
    val progressPercents = mutableListOf<Float>()

    override suspend fun recordOpened(
        documentId: String,
        sessionId: String,
    ) {
        failWith?.let { throw it }
        openedDocuments += documentId
    }

    override suspend fun recordProgress(
        documentId: String,
        percent: Float,
        sessionId: String,
    ) {
        failWith?.let { throw it }
        progressPercents += percent
    }
}
