package app.indelible.core.offline

import kotlinx.coroutines.flow.Flow

/** What a reader needs from the download layer: a copy on request, its progress, and a hold while it is open. */
interface ReadingCopies {
    suspend fun cacheOnOpen(
        session: Session,
        documentId: String,
    )

    suspend fun requestForReading(
        session: Session,
        documentId: String,
    )

    fun observeAcquisitions(scope: String): Flow<Map<String, Acquisition>>

    suspend fun hold(
        scope: String,
        documentId: String,
    )

    suspend fun release(
        scope: String,
        documentId: String,
    )
}
