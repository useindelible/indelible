package app.indelible.reader.repository

import app.indelible.core.model.SaveItemRequest
import app.indelible.core.network.LibraryApiService
import app.indelible.core.network.ReaderApiService
import app.indelible.core.offline.OfflineStore
import app.indelible.core.offline.OutboxWorker
import app.indelible.reader.model.ArticleToc
import app.indelible.reader.model.CreateHighlightRequest
import app.indelible.reader.model.DocumentEntity
import app.indelible.reader.model.HighlightData
import app.indelible.reader.model.HighlightLocator
import app.indelible.reader.model.HighlightNoteData
import app.indelible.reader.model.ReaderDocument
import app.indelible.reader.model.ReaderReprocessResult
import app.indelible.reader.model.TagData
import app.indelible.reader.model.toHighlightData
import app.indelible.reader.model.toHighlightNoteData
import app.indelible.reader.model.toReaderDocument
import app.indelible.reader.model.toTagData
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob

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

    suspend fun updateProgress(
        itemId: String,
        percent: Float,
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
    private val scopeProvider: suspend () -> String?,
    drainScope: CoroutineScope = CoroutineScope(SupervisorJob() + Dispatchers.Default),
) : ReaderRepository,
    ReadingEventWriter {
    private val outbox = ReaderOutboxWrites(offlineStore, worker, drainScope)

    override suspend fun getItem(itemId: String): Result<ReaderDocument> =
        readerApiService.getDocumentReader(itemId).map { it.toReaderDocument() }

    override suspend fun triageItem(
        itemId: String,
        state: String,
    ): Result<Unit> = libraryApiService.triageItem(itemId, state).map {}

    override suspend fun fetchReadableHtml(itemId: String): Result<String> = readerApiService.streamAsset(itemId, "readable_html")

    override suspend fun getArticleToc(itemId: String): Result<ArticleToc> = readerApiService.getArticleToc(itemId)

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

    override suspend fun updateProgress(
        itemId: String,
        percent: Float,
    ): Result<Unit> = readerApiService.updateProgress(itemId, percent)

    override suspend fun recordOpened(
        documentId: String,
        sessionId: String,
    ) {
        recordEvent(documentId, KIND_OPENED, sessionId, progressBasisPoints = null) {
            Result.success(Unit)
        }
    }

    override suspend fun recordProgress(
        documentId: String,
        percent: Float,
        sessionId: String,
    ) {
        recordEvent(documentId, KIND_PROGRESS, sessionId, basisPoints(percent)) {
            readerApiService.updateProgress(documentId, percent)
        }
    }

    /** Recording is best-effort: a reader must keep reading when its event cannot be written. */
    private suspend fun recordEvent(
        documentId: String,
        kind: String,
        sessionId: String,
        progressBasisPoints: Int?,
        withoutScope: suspend () -> Result<Unit>,
    ) {
        val scope = scopeProvider()
        if (scope == null) {
            withoutScope()
            return
        }
        offlineResult { outbox.readingEvent(scope, documentId, kind, sessionId, progressBasisPoints) }
    }

    override suspend fun listHighlights(itemId: String): Result<List<HighlightData>> =
        readerApiService.listHighlights(itemId).map { response -> response.highlights.map { it.toHighlightData() } }

    override suspend fun listDocumentEntities(itemId: String) = readerApiService.listDocumentEntities(itemId)

    override suspend fun createHighlight(
        itemId: String,
        color: String,
        textContent: String,
        startOffset: Long,
        endOffset: Long,
    ): Result<HighlightData> {
        val locator = HighlightLocator(type = "html", startOffset = startOffset, endOffset = endOffset)
        val scope =
            scopeProvider() ?: return readerApiService
                .createHighlight(
                    itemId,
                    CreateHighlightRequest(color = color, textContent = textContent, locator = locator),
                ).map { it.toHighlightData() }
        return offlineResult { outbox.createHighlight(scope, itemId, color, textContent, locator) }
    }

    override suspend fun deleteHighlight(
        itemId: String,
        highlightId: String,
    ): Result<Unit> {
        val scope = scopeProvider() ?: return readerApiService.deleteHighlight(highlightId)
        return offlineResult { outbox.deleteHighlight(scope, itemId, highlightId) }
    }

    override suspend fun updateHighlightColor(
        itemId: String,
        highlightId: String,
        color: String,
    ): Result<HighlightData> {
        val scope =
            scopeProvider() ?: return readerApiService.patchHighlight(highlightId, color).map { it.toHighlightData() }
        return offlineResult { outbox.updateHighlightColor(scope, itemId, highlightId, color) }
    }

    override suspend fun upsertHighlightNote(
        itemId: String,
        highlightId: String,
        body: String,
    ): Result<HighlightNoteData> {
        val scope =
            scopeProvider() ?: return readerApiService
                .upsertHighlightNote(highlightId, body)
                .map { it.toHighlightNoteData() }
        return offlineResult { outbox.upsertHighlightNote(scope, itemId, highlightId, body) }
    }

    override suspend fun deleteHighlightNote(
        itemId: String,
        highlightId: String,
    ): Result<Unit> {
        val scope = scopeProvider() ?: return readerApiService.deleteHighlightNote(highlightId)
        return offlineResult { outbox.deleteHighlightNote(scope, itemId, highlightId) }
    }

    override suspend fun setHighlightTags(
        itemId: String,
        highlightId: String,
        tags: List<String>,
    ): Result<List<String>> {
        val scope = scopeProvider() ?: return readerApiService.setHighlightTags(highlightId, tags)
        return offlineResult {
            outbox.setHighlightTags(scope, itemId, highlightId, tags)
            tags
        }
    }

    override suspend fun listTags(): Result<List<TagData>> = readerApiService.listTags().map { tags -> tags.map { it.toTagData() } }

    override suspend fun getItemNote(itemId: String): Result<String?> = readerApiService.getItemNote(itemId).map { it?.body }

    override suspend fun upsertItemNote(
        itemId: String,
        body: String,
    ): Result<String> {
        val scope = scopeProvider() ?: return readerApiService.upsertItemNote(itemId, body).map { it.body }
        return offlineResult {
            outbox.upsertDocumentNote(scope, itemId, body)
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
