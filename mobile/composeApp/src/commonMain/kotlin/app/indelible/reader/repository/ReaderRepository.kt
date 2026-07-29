package app.indelible.reader.repository

import app.indelible.core.model.SaveItemRequest
import app.indelible.core.network.LibraryApiService
import app.indelible.core.network.ReaderApiService
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

    suspend fun deleteHighlight(highlightId: String): Result<Unit>

    suspend fun updateHighlightColor(
        highlightId: String,
        color: String,
    ): Result<HighlightData>

    suspend fun upsertHighlightNote(
        highlightId: String,
        body: String,
    ): Result<HighlightNoteData>

    suspend fun deleteHighlightNote(highlightId: String): Result<Unit>

    suspend fun setHighlightTags(
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

class ApiReaderRepository(
    private val readerApiService: ReaderApiService,
    private val libraryApiService: LibraryApiService,
) : ReaderRepository {
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

    override suspend fun listHighlights(itemId: String): Result<List<HighlightData>> =
        readerApiService.listHighlights(itemId).map { response -> response.highlights.map { it.toHighlightData() } }

    override suspend fun listDocumentEntities(itemId: String) = readerApiService.listDocumentEntities(itemId)

    override suspend fun createHighlight(
        itemId: String,
        color: String,
        textContent: String,
        startOffset: Long,
        endOffset: Long,
    ): Result<HighlightData> =
        readerApiService
            .createHighlight(
                itemId,
                CreateHighlightRequest(
                    color = color,
                    textContent = textContent,
                    locator =
                        HighlightLocator(
                            type = "html",
                            startOffset = startOffset,
                            endOffset = endOffset,
                        ),
                ),
            ).map { it.toHighlightData() }

    override suspend fun deleteHighlight(highlightId: String): Result<Unit> = readerApiService.deleteHighlight(highlightId)

    override suspend fun updateHighlightColor(
        highlightId: String,
        color: String,
    ): Result<HighlightData> = readerApiService.patchHighlight(highlightId, color).map { it.toHighlightData() }

    override suspend fun upsertHighlightNote(
        highlightId: String,
        body: String,
    ): Result<HighlightNoteData> = readerApiService.upsertHighlightNote(highlightId, body).map { it.toHighlightNoteData() }

    override suspend fun deleteHighlightNote(highlightId: String): Result<Unit> = readerApiService.deleteHighlightNote(highlightId)

    override suspend fun setHighlightTags(
        highlightId: String,
        tags: List<String>,
    ): Result<List<String>> = readerApiService.setHighlightTags(highlightId, tags)

    override suspend fun listTags(): Result<List<TagData>> = readerApiService.listTags().map { tags -> tags.map { it.toTagData() } }

    override suspend fun getItemNote(itemId: String): Result<String?> = readerApiService.getItemNote(itemId).map { it?.body }

    override suspend fun upsertItemNote(
        itemId: String,
        body: String,
    ): Result<String> = readerApiService.upsertItemNote(itemId, body).map { it.body }

    override suspend fun getItemTags(itemId: String): Result<List<String>> = readerApiService.getItemTags(itemId)

    override suspend fun setItemTags(
        itemId: String,
        tags: List<String>,
    ): Result<List<String>> = readerApiService.setItemTags(itemId, tags)
}
