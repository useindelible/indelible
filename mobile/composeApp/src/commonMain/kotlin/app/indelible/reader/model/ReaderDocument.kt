package app.indelible.reader.model

import app.indelible.api.generated.models.DocumentReaderResponse
import kotlinx.datetime.Instant

data class ReaderDocument(
    val id: String,
    val itemType: String,
    val title: String,
    val url: String? = null,
    val saved: Boolean,
    val readableReady: Boolean,
    val availableAssets: List<String>,
    val libraryEntryId: String? = null,
    val progressPercent: Int? = null,
    val maxProgressPercent: Int? = null,
    val lastReadAt: Instant? = null,
    val finishedAt: Instant? = null,
    val language: String? = null,
    val chapterLocator: String? = null,
    val chapterOffset: Int? = null,
    val triageState: String? = if (saved) "inbox" else null,
    val canonicalUrl: String? = url,
    val thumbnailUrl: String? = null,
    val leadImageUrl: String? = null,
    val author: String? = null,
    val videoDurationSeconds: Int? = null,
    val domain: String? = null,
    val publishedAt: Instant? = null,
    val readingTimeMinutes: Int? = null,
    val summary: String? = null,
    val wordCount: Int? = null,
    val savedAt: Instant = lastReadAt ?: Instant.parse("1970-01-01T00:00:00Z"),
) {
    val documentId: String get() = id

    val documentType: String get() = itemType
}

fun DocumentReaderResponse.toReaderDocument(): ReaderDocument =
    ReaderDocument(
        id = documentId,
        itemType = documentType,
        title = title,
        url = url,
        saved = saved,
        readableReady = readableReady,
        availableAssets = availableAssets,
        libraryEntryId = libraryEntryId,
        progressPercent = progressPercent,
        maxProgressPercent = maxProgressPercent,
        lastReadAt = lastReadAt,
        finishedAt = finishedAt,
        language = language,
        chapterLocator = chapterLocator,
        chapterOffset = chapterOffset,
        triageState = if (saved) "inbox" else null,
        canonicalUrl = url,
        thumbnailUrl = thumbnailUrl,
        leadImageUrl = leadImageUrl,
        author = author,
        domain = domain ?: url?.substringAfter("://")?.substringBefore("/"),
        publishedAt = publishedAt,
        readingTimeMinutes = readingTimeMinutes,
        summary = summary ?: excerpt,
        wordCount = wordCount,
        savedAt = lastReadAt ?: Instant.parse("1970-01-01T00:00:00Z"),
    )
