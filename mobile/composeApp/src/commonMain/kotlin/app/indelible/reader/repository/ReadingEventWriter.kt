package app.indelible.reader.repository

import app.indelible.core.offline.ChapterPosition

/**
 * Reading events carry the session that groups one continuous sitting, which only the open reader
 * knows. [ReaderRepository]'s writes are document-scoped and cannot express it, so a reader records
 * its own events here and keeps its session id out of every other caller's way.
 *
 * Recording is best-effort: an implementation reports what it can and never throws, because a
 * reader that cannot record where it is must still show what it is reading.
 */
interface ReadingEventWriter {
    suspend fun recordOpened(
        documentId: String,
        sessionId: String,
    )

    /** [percent] is `0..100` through the document; [position] is where a book was left, when it is one. */
    suspend fun recordProgress(
        documentId: String,
        percent: Float,
        sessionId: String,
        position: ChapterPosition? = null,
    )
}
