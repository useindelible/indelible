package app.indelible.library.ui.components

import app.indelible.core.model.LibraryCounts
import app.indelible.library.viewmodel.ContentTypeFilter
import kotlin.test.Test
import kotlin.test.assertEquals

class ContentTypeFilterRowTest {
    private val counts =
        LibraryCounts(
            total = 66,
            unread = 42,
            reading = 5,
            done = 19,
            byItemType = mapOf("article" to 38, "video" to 11),
        )

    @Test
    fun visible_filters_drop_types_with_nothing_saved() {
        assertEquals(
            listOf(ContentTypeFilter.ALL, ContentTypeFilter.ARTICLES, ContentTypeFilter.VIDEOS),
            visibleContentTypeFilters(ContentTypeFilter.ALL, counts),
        )
    }

    @Test
    fun visible_filters_keep_the_active_type_even_when_empty() {
        assertEquals(
            listOf(
                ContentTypeFilter.ALL,
                ContentTypeFilter.ARTICLES,
                ContentTypeFilter.PDFS,
                ContentTypeFilter.VIDEOS,
            ),
            visibleContentTypeFilters(ContentTypeFilter.PDFS, counts),
        )
    }

    @Test
    fun visible_filters_show_every_type_until_counts_load() {
        assertEquals(
            ContentTypeFilter.entries,
            visibleContentTypeFilters(ContentTypeFilter.ALL, counts = null),
        )
    }

    @Test
    fun all_chip_reads_the_scope_total_and_missing_types_read_zero() {
        assertEquals(66, counts.countFor(ContentTypeFilter.ALL))
        assertEquals(38, counts.countFor(ContentTypeFilter.ARTICLES))
        assertEquals(0, counts.countFor(ContentTypeFilter.PODCASTS))
    }
}
