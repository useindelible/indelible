package app.indelible.reader.viewmodel

import app.indelible.reader.model.ArticleTocEntry
import kotlin.math.ceil

/**
 * Pure helpers for the Contents panel. Active-section tracking derives from the
 * scroll percentage the WebView already reports plus the per-section word
 * counts the outline carries, so no additional bridge traffic is needed.
 */
object TocSections {
    /** Matches the backend's reading_time_minutes convention. */
    private const val WORDS_PER_MINUTE = 238.0
    private const val FULL_PERCENT = 100f

    fun sectionMinutes(wordCount: Int): Int = maxOf(1, ceil(wordCount / WORDS_PER_MINUTE).toInt())

    /**
     * Index of the section the reader is inside at [scrollPercent], mapping the
     * percentage through cumulative word-count boundaries. Sections without
     * words still occupy a point on the line; a wordless outline degrades to
     * the first section rather than none.
     */
    fun activeSectionIndex(
        scrollPercent: Float,
        entries: List<ArticleTocEntry>,
    ): Int {
        if (entries.isEmpty()) return -1
        val totalWords = entries.sumOf { it.wordCount.toLong() }
        var active = entries.lastIndex
        if (totalWords > 0L) {
            var cumulative = 0L
            val firstInside =
                entries.indexOfFirst { entry ->
                    cumulative += entry.wordCount.toLong()
                    scrollPercent < cumulative * FULL_PERCENT / totalWords
                }
            if (firstInside >= 0) active = firstInside
        } else {
            active = 0
        }
        return active
    }
}
