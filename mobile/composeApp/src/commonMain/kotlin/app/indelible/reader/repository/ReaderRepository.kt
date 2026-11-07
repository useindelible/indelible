package app.indelible.reader.repository

import app.indelible.core.model.SaveItemRequest
import app.indelible.core.network.LibraryApiService
import app.indelible.core.network.ReaderApiService
import app.indelible.core.offline.NoSessionException
import app.indelible.core.offline.OfflineStore
import app.indelible.core.offline.OutboxWorker
import app.indelible.core.offline.Session
import app.indelible.reader.model.ArticleToc
import app.indelible.reader.model.DocumentEntity
import app.indelible.reader.model.HighlightData
import app.indelible.reader.model.HighlightLocator
import app.indelible.reader.model.HighlightNoteData
import app.indelible.reader.model.ReaderDocument
import app.indelible.reader.model.ReaderReprocessResult
import app.indelible.reader.model.TagData
import app.indelible.reader.model.toTagData
import kotlinx.coroutines.CancellationException

interface ReaderRepository {
    suspend fun getItem(itemId: String): Result<ReaderDocument>

    suspend fun triageItem(
        itemId: String,
        state: String,
    ): Result<Unit>

    suspend fun fetchReadableHtml(itemId: String): Result<String>

    suspend fun getArticleToc(itemId: String): Result<ArticleToc>

    suspend fun reprocessDocument(itemId: String): Result<ReaderReprocessResult>

    suspend fun saveToLibrary(
        url: String,
        title: String?,
        itemType: String?,
    ): Result<Unit>

    suspend fun listHighlights(itemId: String): Result<List<HighlightData>>

    suspend fun listDocumentEntities(itemId: String): Result<List<DocumentEntity>>

    suspend fun createHighlight(
        itemId: String,
        color: String,
        textContent: String,
        startOffset: Long,
        endOffset: Long,
    ): Result<HighlightData>

    suspend fun deleteHighlight(
        itemId: String,
        highlightId: String,
    ): Result<Unit>

    suspend fun updateHighlightColor(
        itemId: String,
        highlightId: String,
        color: String,
    ): Result<HighlightData>

    suspend fun upsertHighlightNote(
        itemId: String,
        highlightId: String,
        body: String,
    ): Result<HighlightNoteData>

    suspend fun deleteHighlightNote(
        itemId: String,
        highlightId: String,
    ): Result<Unit>

    suspend fun setHighlightTags(
        itemId: String,
        highlightId: String,
        tags: List<String>,
    ): Result<List<String>>

    suspend fun listTags(): Result<List<TagData>>

    suspend fun getItemNote(itemId: String): Result<String?>

    suspend fun upsertItemNote(
        itemId: String,
        body: String,
    ): Result<String>

    suspend fun getItemTags(itemId: String): Result<List<String>>

    suspend fun setItemTags(
        itemId: String,
        tags: List<String>,
    ): Result<List<String>>
}

private const val KIND_OPENED = "opened"
private const val KIND_PROGRESS = "progress"

class ApiReaderRepository(
    private val readerApiService: ReaderApiService,
    private val libraryApiService: LibraryApiService,
    offlineStore: OfflineStore,
    worker: OutboxWorker,
    private val sessionProvider: () -> Session?,
    private val offlineCopy: ReaderOfflineCopy,
) : ReaderRepository,
    ReadingEventWriter {
    private val outbox = ReaderOutboxWrites(offlineStore, worker)

    override suspend fun getItem(itemId: String): Result<ReaderDocument> =
        offlineCopy.document(itemId) { readerApiService.getDocumentReader(itemId) }

    override suspend fun triageItem(
        itemId: String,
        state: String,
    ): Result<Unit> = libraryApiService.triageItem(itemId, state).map {}

    override suspend fun fetchReadableHtml(itemId: String): Result<String> =
        offlineCopy.readableHtml(itemId) { readerApiService.streamAsset(itemId, "readable_html") }

    override suspend fun getArticleToc(itemId: String): Result<ArticleToc> =
        offlineCopy.articleToc(itemId) { readerApiService.getArticleToc(itemId) }

    override suspend fun reprocessDocument(itemId: String): Result<ReaderReprocessResult> =
        readerApiService.reprocessDocument(itemId).map { response ->
            ReaderReprocessResult(
                queued = response.queued,
                retryAfterSeconds = response.retryAfterSeconds,
            )
        }

    /** Saves the document to the library by URL, mirroring web's create-document-entry flow. */
    override suspend fun saveToLibrary(
        url: String,
        title: String?,
        itemType: String?,
    ): Result<Unit> =
        libraryApiService
            .saveItem(SaveItemRequest(url = url, title = title, itemType = itemType))
            .map {}

    override suspend fun recordOpened(
        documentId: String,
        sessionId: String,
    ) {
        recordEvent(documentId, KIND_OPENED, sessionId, progressBasisPoints = null)
    }

    override suspend fun recordProgress(
        documentId: String,
        percent: Float,
        sessionId: String,
    ) {
        recordEvent(documentId, KIND_PROGRESS, sessionId, basisPoints(percent))
    }

    /** Recording is best-effort: a reader must keep reading when its event cannot be written. */
    private suspend fun recordEvent(
        documentId: String,
        kind: String,
        sessionId: String,
        progressBasisPoints: Int?,
    ) {
        val session = sessionProvider() ?: return
        offlineResult { outbox.readingEvent(session, documentId, kind, sessionId, progressBasisPoints) }
    }

    override suspend fun listHighlights(itemId: String): Result<List<HighlightData>> =
        offlineCopy.highlights(itemId) { readerApiService.listHighlights(itemId) }

    override suspend fun listDocumentEntities(itemId: String) = readerApiService.listDocumentEntities(itemId)

    override suspend fun createHighlight(
        itemId: String,
        color: String,
        textContent: String,
        startOffset: Long,
        endOffset: Long,
    ): Result<HighlightData> {
        val locator = HighlightLocator(type = "html", startOffset = startOffset, endOffset = endOffset)
        val session = sessionProvider() ?: return Result.failure(NoSessionException())
        return offlineResult { outbox.createHighlight(session, itemId, color, textContent, locator) }
    }

    override suspend fun deleteHighlight(
        itemId: String,
        highlightId: String,
    ): Result<Unit> {
        val session = sessionProvider() ?: return Result.failure(NoSessionException())
        return offlineResult { outbox.deleteHighlight(session, itemId, highlightId) }
    }

    override suspend fun updateHighlightColor(
        itemId: String,
        highlightId: String,
        color: String,
    ): Result<HighlightData> {
        val session = sessionProvider() ?: return Result.failure(NoSessionException())
        return offlineResult { outbox.updateHighlightColor(session, itemId, highlightId, color) }
    }

    override suspend fun upsertHighlightNote(
        itemId: String,
        highlightId: String,
        body: String,
    ): Result<HighlightNoteData> {
        val session = sessionProvider() ?: return Result.failure(NoSessionException())
        return offlineResult { outbox.upsertHighlightNote(session, itemId, highlightId, body) }
    }

    override suspend fun deleteHighlightNote(
        itemId: String,
        highlightId: String,
    ): Result<Unit> {
        val session = sessionProvider() ?: return Result.failure(NoSessionException())
        return offlineResult { outbox.deleteHighlightNote(session, itemId, highlightId) }
    }

    override suspend fun setHighlightTags(
        itemId: String,
        highlightId: String,
        tags: List<String>,
    ): Result<List<String>> {
        val session = sessionProvider() ?: return Result.failure(NoSessionException())
        return offlineResult {
            outbox.setHighlightTags(session, itemId, highlightId, tags)
            tags
        }
    }

    override suspend fun listTags(): Result<List<TagData>> = readerApiService.listTags().map { tags -> tags.map { it.toTagData() } }

    override suspend fun getItemNote(itemId: String): Result<String?> =
        offlineCopy.note(itemId) {
            readerApiService.getItemNote(itemId)
        }

    override suspend fun upsertItemNote(
        itemId: String,
        body: String,
    ): Result<String> {
        val session = sessionProvider() ?: return Result.failure(NoSessionException())
        return offlineResult {
            outbox.upsertDocumentNote(session, itemId, body)
            body
        }
    }

    override suspend fun getItemTags(itemId: String): Result<List<String>> = readerApiService.getItemTags(itemId)

    override suspend fun setItemTags(
        itemId: String,
        tags: List<String>,
    ): Result<List<String>> = readerApiService.setItemTags(itemId, tags)

    /**
     * A local write fails the way a network write does, so callers keep their existing failure
     * handling; cancellation stays a cancellation.
     */
    private suspend fun <T> offlineResult(write: suspend () -> T): Result<T> {
        val result = runCatching { write() }
        result.exceptionOrNull()?.let { if (it is CancellationException) throw it }
        return result
    }
}
