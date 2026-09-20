package app.indelible.core.offline

data class ServerDocument(
    val title: String,
    val readerJson: String,
    val progress: Progress,
    val highlights: List<CachedHighlight>,
    val note: ServerNote?,
)

data class ServerNote(
    val body: String,
    val updatedAtEpochMs: Long,
)

/** A complete copy to install; [revision] is the one read before the fetch began. */
data class InstallRequest(
    val documentId: String,
    val documentType: String,
    val revision: DocumentRevision,
    val server: ServerDocument,
    val generation: Long,
    val assets: List<CachedAssetRow>,
    val bytes: Long,
    val pin: Boolean,
    val at: Long,
)

sealed interface ServerPart {
    /** The title and reader JSON always apply; the progress is a position part. */
    data class Reader(
        val title: String,
        val readerJson: String,
        val progress: Progress,
    ) : ServerPart

    data class Highlights(
        val highlights: List<CachedHighlight>,
    ) : ServerPart

    data class Note(
        val note: ServerNote?,
    ) : ServerPart
}

data class RefreshRequest(
    val documentId: String,
    val revision: DocumentRevision,
    val part: ServerPart,
    val at: Long,
)

/** Which parts of a server response predate a local change: content is highlights and note. */
data class Staleness(
    val content: Boolean,
    val position: Boolean,
)

sealed interface InstallResult {
    data object Installed : InstallResult

    /** Nothing was written: an install is all or nothing. */
    data class Stale(
        val content: Boolean,
        val position: Boolean,
    ) : InstallResult
}

sealed interface RefreshResult {
    data object NoCopy : RefreshResult

    data object Applied : RefreshResult

    /** The part predates a local change and was not applied; of a reader part, only its progress. */
    data object Stale : RefreshResult
}

data class CatalogEntry(
    val documentId: String,
    val documentType: String,
    val title: String,
    val pinned: Boolean,
    val lastOpenedAt: Long,
    val lastSyncedAt: Long?,
    val bytes: Long,
    val generation: Long,
)

/** Unsuperseded outbox rows of one document; [retrying] is the part of [pending] already attempted. */
data class DocumentSyncCounts(
    val pending: Int,
    val retrying: Int,
    val failed: Int,
    val blocked: Int,
)
