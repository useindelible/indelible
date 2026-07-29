package app.indelible.core.model

import app.indelible.api.generated.models.LibraryEntryResponse
import app.indelible.api.generated.models.PaginatedResponseLibraryEntryResponse
import kotlinx.datetime.Instant

data class LibraryItem(
    val id: String,
    val documentId: String,
    val itemType: String,
    val title: String,
    val url: String? = null,
    val canonicalUrl: String? = null,
    val source: String,
    val domain: String? = null,
    val author: String? = null,
    val publishedAt: Instant? = null,
    val savedAt: Instant,
    val createdAt: Instant,
    val updatedAt: Instant,
    val triageState: String,
    val isFavorite: Boolean,
    val isShortlisted: Boolean,
    val language: String? = null,
    val excerpt: String? = null,
    val summary: String? = excerpt,
    val thumbnailUrl: String? = null,
    val leadImageUrl: String? = null,
    val readingTimeMinutes: Int? = null,
    val wordCount: Int? = null,
    val sourceDeliveryId: String? = null,
    val isUnread: Boolean = true,
    val progressPercent: Float? = null,
    val lastReadAt: Instant? = null,
    val pipelineStatus: String? = null,
    val pipelineError: String? = null,
    val videoDurationSeconds: Int? = null,
    val deletedAt: Instant? = null,
)

data class PaginatedItems(
    val `data`: List<LibraryItem>,
    val page: PageInfo,
)

fun LibraryEntryResponse.toLibraryItem(): LibraryItem =
    LibraryItem(
        id = libraryEntryId,
        documentId = documentId,
        itemType = documentType,
        title = title,
        url = url,
        canonicalUrl = canonicalUrl,
        source = source,
        domain = domain,
        author = author,
        publishedAt = publishedAt,
        savedAt = savedAt,
        createdAt = createdAt,
        updatedAt = updatedAt,
        triageState = triageState,
        isFavorite = isFavorite,
        isShortlisted = isShortlisted,
        language = language,
        excerpt = excerpt,
        summary = summary ?: excerpt,
        thumbnailUrl = thumbnailUrl,
        leadImageUrl = leadImageUrl,
        readingTimeMinutes = readingTimeMinutes,
        wordCount = wordCount,
        sourceDeliveryId = sourceDeliveryId,
        deletedAt = null,
    )

fun PaginatedResponseLibraryEntryResponse.toPaginatedItems(): PaginatedItems =
    PaginatedItems(
        data = data.map { it.toLibraryItem() },
        page = page,
    )

/**
 * Whole minutes of reading remaining, derived from the total reading-time
 * estimate and how far through the item the reader is. Returns null when there
 * is no estimate or the item is effectively finished, so callers can simply omit
 * the "MIN LEFT" label rather than render "0 MIN LEFT". Floors at one minute so a
 * nearly-complete item still reads as having a sliver left.
 */
fun LibraryItem.readingMinutesLeft(): Int? {
    return null
}

fun LibraryItem.withTriageState(state: String): LibraryItem {
    return copy(triageState = state)
}

/**
 * Formats a minute count as an uppercase reading-time label, rolling past an hour
 * so long reads stay legible: 8 -> "8 MIN", 60 -> "1 HR", 90 -> "1 HR 30 MIN".
 * Shared by the list eyebrow (total time) and the progress label (time left), so
 * both read consistently.
 */
private const val MINUTES_PER_HOUR = 60
private const val HASH_PRIME = 31

fun formatReadingTime(minutes: Int): String {
    val hours = minutes / MINUTES_PER_HOUR
    val mins = minutes % MINUTES_PER_HOUR
    return when {
        hours == 0 -> "$mins MIN"
        mins == 0 -> "$hours HR"
        else -> "$hours HR $mins MIN"
    }
}

enum class ThumbnailColor {
    BLUE,
    GREEN,
    PURPLE,
    ORANGE,
    RED,
    TEAL,
    PINK,
    ;

    companion object {
        fun forId(id: String): ThumbnailColor {
            var hash = 0
            for (ch in id) {
                hash = hash * HASH_PRIME + ch.code
            }
            val index = ((hash % entries.size) + entries.size) % entries.size
            return entries[index]
        }
    }
}
