package app.indelible.reader.viewmodel

import app.indelible.reader.model.ArticleToc
import app.indelible.reader.model.ArticleTocStatus
import app.indelible.reader.model.DocumentEntity
import app.indelible.reader.model.HighlightData
import app.indelible.reader.model.HighlightLocator
import app.indelible.reader.model.HighlightNoteData
import app.indelible.reader.model.ReaderDocument
import app.indelible.reader.model.ReaderReprocessResult
import app.indelible.reader.model.TagData
import app.indelible.reader.repository.ReaderRepository
import kotlinx.datetime.Instant

class FakeReaderRepository : ReaderRepository {
    var getItemResult: Result<ReaderDocument> = Result.success(fakeItemDetail())

    /** When set, successive getItem calls return these in order, repeating the last entry. */
    var getItemResults: List<Result<ReaderDocument>>? = null
    var getItemCallCount = 0
    var triageItemResult: Result<Unit> = Result.success(Unit)
    var fetchHtmlResult: Result<String> = Result.success("<p>Hello world</p>")
    var getArticleTocResult: Result<ArticleToc> =
        Result.success(ArticleToc(entries = emptyList(), status = ArticleTocStatus.NONE, truncated = false))

    /** When set, successive getArticleToc calls return these in order, repeating the last entry. */
    var getArticleTocResults: List<Result<ArticleToc>>? = null
    var getArticleTocCallCount = 0
    var updateProgressResult: Result<Unit> = Result.success(Unit)
    var listHighlightsResult: Result<List<HighlightData>> = Result.success(emptyList())
    var listDocumentEntitiesResult: Result<List<DocumentEntity>> = Result.success(emptyList())
    var createHighlightResult: Result<HighlightData> = Result.success(fakeHighlight())
    var deleteHighlightResult: Result<Unit> = Result.success(Unit)
    var updateHighlightColorResult: Result<HighlightData> = Result.success(fakeHighlight())
    var upsertNoteResult: Result<HighlightNoteData> = Result.success(fakeNote())
    var deleteNoteResult: Result<Unit> = Result.success(Unit)
    var setHighlightTagsResult: Result<List<String>> = Result.success(emptyList())
    var listTagsResult: Result<List<TagData>> = Result.success(emptyList())
    var reprocessDocumentResult: Result<ReaderReprocessResult> =
        Result.success(ReaderReprocessResult(queued = true, retryAfterSeconds = null))
    val contentCallLog = mutableListOf<String>()

    var saveToLibraryResult: Result<Unit> = Result.success(Unit)
    var lastSavedUrl: String? = null

    var getItemNoteResult: Result<String?> = Result.success(null)
    var upsertItemNoteResult: Result<String>? = null
    var getItemTagsResult: Result<List<String>> = Result.success(emptyList())
    var setItemTagsResult: Result<List<String>> = Result.success(emptyList())

    var lastProgressItemId: String? = null
    var lastProgressPercent: Float? = null
    var createHighlightCallCount = 0
    var deleteHighlightCallCount = 0
    var lastDeletedHighlightId: String? = null
    var lastUpsertedNote: String? = null
    var lastSetItemTags: List<String>? = null
    var lastTriagedState: String? = null
    var lastGetItemTagsId: String? = null
    var lastSetItemTagsId: String? = null

    override suspend fun getItem(itemId: String): Result<ReaderDocument> {
        contentCallLog.add("getItem:$itemId")
        val seq = getItemResults
        val result =
            if (seq != null && seq.isNotEmpty()) {
                seq[minOf(getItemCallCount, seq.lastIndex)]
            } else {
                getItemResult
            }
        getItemCallCount++
        return result
    }

    override suspend fun triageItem(
        itemId: String,
        state: String,
    ): Result<Unit> {
        lastTriagedState = state
        return triageItemResult
    }

    override suspend fun fetchReadableHtml(itemId: String): Result<String> {
        contentCallLog.add("fetchReadableHtml:$itemId")
        return fetchHtmlResult
    }

    override suspend fun getArticleToc(itemId: String): Result<ArticleToc> {
        val results = getArticleTocResults
        val result =
            if (results != null) {
                results[minOf(getArticleTocCallCount, results.lastIndex)]
            } else {
                getArticleTocResult
            }
        getArticleTocCallCount += 1
        return result
    }

    override suspend fun reprocessDocument(itemId: String): Result<ReaderReprocessResult> {
        contentCallLog.add("reprocessDocument:$itemId")
        return reprocessDocumentResult
    }

    override suspend fun saveToLibrary(
        url: String,
        title: String?,
        itemType: String?,
    ): Result<Unit> {
        lastSavedUrl = url
        return saveToLibraryResult
    }

    override suspend fun updateProgress(
        itemId: String,
        percent: Float,
    ): Result<Unit> {
        lastProgressItemId = itemId
        lastProgressPercent = percent
        return updateProgressResult
    }

    override suspend fun listHighlights(itemId: String): Result<List<HighlightData>> = listHighlightsResult

    override suspend fun listDocumentEntities(itemId: String): Result<List<DocumentEntity>> = listDocumentEntitiesResult

    override suspend fun createHighlight(
        itemId: String,
        color: String,
        textContent: String,
        startOffset: Long,
        endOffset: Long,
    ): Result<HighlightData> {
        createHighlightCallCount++
        return createHighlightResult
    }

    override suspend fun deleteHighlight(highlightId: String): Result<Unit> {
        deleteHighlightCallCount++
        lastDeletedHighlightId = highlightId
        return deleteHighlightResult
    }

    override suspend fun updateHighlightColor(
        highlightId: String,
        color: String,
    ): Result<HighlightData> = updateHighlightColorResult

    override suspend fun upsertHighlightNote(
        highlightId: String,
        body: String,
    ): Result<HighlightNoteData> = upsertNoteResult

    override suspend fun deleteHighlightNote(highlightId: String): Result<Unit> = deleteNoteResult

    override suspend fun setHighlightTags(
        highlightId: String,
        tags: List<String>,
    ): Result<List<String>> = setHighlightTagsResult

    override suspend fun listTags(): Result<List<TagData>> = listTagsResult

    override suspend fun getItemNote(itemId: String): Result<String?> = getItemNoteResult

    override suspend fun upsertItemNote(
        itemId: String,
        body: String,
    ): Result<String> {
        lastUpsertedNote = body
        return upsertItemNoteResult ?: Result.success(body)
    }

    override suspend fun getItemTags(itemId: String): Result<List<String>> {
        lastGetItemTagsId = itemId
        return getItemTagsResult
    }

    override suspend fun setItemTags(
        itemId: String,
        tags: List<String>,
    ): Result<List<String>> {
        lastSetItemTagsId = itemId
        lastSetItemTags = tags
        return setItemTagsResult
    }

    companion object {
        fun fakeItemDetail(
            id: String = "lib_test1",
            documentId: String = "doc_test1",
            progressPercent: Float? = null,
            maxProgressPercent: Float? = null,
            finishedAt: Instant? = null,
            url: String? = "https://example.com/article",
            itemType: String = "article",
            saved: Boolean = true,
            readableReady: Boolean = true,
        ) = ReaderDocument(
            id = documentId,
            itemType = itemType,
            title = "Test Article",
            libraryEntryId = if (saved) id else null,
            url = url,
            saved = saved,
            readableReady = readableReady,
            progressPercent = progressPercent?.toInt(),
            maxProgressPercent = maxProgressPercent?.toInt(),
            finishedAt = finishedAt,
            availableAssets = listOf("readable_html"),
            lastReadAt = Instant.parse("2024-01-01T00:00:00Z"),
        )

        fun fakeHighlight(
            id: String = "hlt_test1",
            color: String = "yellow",
            textContent: String = "highlighted text",
        ) = HighlightData(
            id = id,
            documentId = "doc_test1",
            color = color,
            textContent = textContent,
            locator = HighlightLocator(type = "html", startOffset = 0, endOffset = 15),
            tags = emptyList(),
            createdAt = Instant.parse("2024-01-01T00:00:00Z"),
            updatedAt = Instant.parse("2024-01-01T00:00:00Z"),
        )

        fun fakeNote(
            id: String = "hln_test1",
            highlightId: String = "hlt_test1",
        ) = HighlightNoteData(
            id = id,
            highlightId = highlightId,
            body = "test note",
            createdAt = Instant.parse("2024-01-01T00:00:00Z"),
            updatedAt = Instant.parse("2024-01-01T00:00:00Z"),
        )
    }
}
