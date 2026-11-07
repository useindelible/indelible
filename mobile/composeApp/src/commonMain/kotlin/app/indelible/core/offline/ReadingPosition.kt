package app.indelible.core.offline

sealed interface ReadingPosition {
    val locator: String
    val offset: Int
}

data class ChapterPosition(
    val chapter: String,
    override val offset: Int,
) : ReadingPosition {
    override val locator: String get() = chapter
}

/** [page] is 1-based; [offset] is 0..[PAGE_OFFSET_SCALE] of the page height from its top. */
data class PagePosition(
    val page: Int,
    override val offset: Int,
) : ReadingPosition {
    override val locator: String get() = "$PAGE_PREFIX$page"
}

const val PAGE_OFFSET_SCALE = 10_000

private const val PAGE_PREFIX = "page:"

/** The server's rule: `page:N` with N >= 1 is a page, anything else a chapter key. */
fun readingPositionOf(
    locator: String,
    offset: Int,
): ReadingPosition {
    val page = locator.removePrefix(PAGE_PREFIX).takeIf { it != locator }?.toIntOrNull()
    return if (page != null && page >= 1) PagePosition(page, offset) else ChapterPosition(locator, offset)
}
