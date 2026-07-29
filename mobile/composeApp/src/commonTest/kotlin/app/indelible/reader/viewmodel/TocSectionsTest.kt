package app.indelible.reader.viewmodel

import app.indelible.reader.model.ArticleTocEntry
import kotlin.test.Test
import kotlin.test.assertEquals

class TocSectionsTest {
    private fun entry(
        index: Int,
        words: Int,
    ) = ArticleTocEntry(
        depth = 0,
        id = "ind-toc-$index",
        sourceHeadingIndex = index,
        title = "Section $index",
        wordCount = words,
    )

    @Test
    fun active_section_maps_scroll_percent_through_cumulative_word_counts() {
        // Boundaries at 10% / 40% / 100% of the sectioned words.
        val entries = listOf(entry(0, 100), entry(1, 300), entry(2, 600))
        assertEquals(0, TocSections.activeSectionIndex(0f, entries))
        assertEquals(0, TocSections.activeSectionIndex(9f, entries))
        assertEquals(1, TocSections.activeSectionIndex(35f, entries))
        assertEquals(2, TocSections.activeSectionIndex(95f, entries))
        assertEquals(2, TocSections.activeSectionIndex(100f, entries))
    }

    @Test
    fun active_section_handles_empty_and_zero_word_lists() {
        assertEquals(-1, TocSections.activeSectionIndex(50f, emptyList()))
        val zeroWords = listOf(entry(0, 0), entry(1, 0))
        assertEquals(0, TocSections.activeSectionIndex(50f, zeroWords))
    }

    @Test
    fun section_minutes_round_up_at_238_wpm_with_a_floor_of_one() {
        assertEquals(1, TocSections.sectionMinutes(0))
        assertEquals(1, TocSections.sectionMinutes(238))
        assertEquals(2, TocSections.sectionMinutes(239))
        assertEquals(3, TocSections.sectionMinutes(600))
    }
}
