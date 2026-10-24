package app.indelible.reader.repository

import app.indelible.core.offline.ChapterPosition

/** The open reader's own event writer, keyed by its sitting's session id; best-effort, never throws. */
interface ReadingEventWriter {
    suspend fun recordOpened(
        documentId: String,
        sessionId: String,
    )

    suspend fun recordProgress(
        documentId: String,
        percent: Float,
        sessionId: String,
        position: ChapterPosition? = null,
    )
}
