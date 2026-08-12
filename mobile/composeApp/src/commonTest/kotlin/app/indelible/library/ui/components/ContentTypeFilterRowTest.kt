package app.indelible.library.ui.components

import app.indelible.core.model.LibraryCounts
import app.indelible.library.viewmodel.ContentTypeFilter
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse

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
            ContentTypeFilter.entries.filterNot { it == ContentTypeFilter.PODCASTS },
            visibleContentTypeFilters(ContentTypeFilter.ALL, counts = null),
        )
    }

    @Test
    fun visible_filters_keep_the_full_layout_when_the_scope_is_empty() {
        val emptyCounts =
            LibraryCounts(
                total = 0,
                unread = 0,
                reading = 0,
                done = 0,
                byItemType = emptyMap(),
            )

        assertEquals(
            ContentTypeFilter.entries.filterNot { it == ContentTypeFilter.PODCASTS },
            visibleContentTypeFilters(ContentTypeFilter.ALL, emptyCounts),
        )
    }

    @Test
    fun visible_filters_never_advertise_podcasts_at_launch() {
        val podcastCounts =
            LibraryCounts(
                total = 3,
                unread = 3,
                reading = 0,
                done = 0,
                byItemType = mapOf("podcast" to 3),
            )

        assertFalse(
            visibleContentTypeFilters(ContentTypeFilter.PODCASTS, podcastCounts)
                .contains(ContentTypeFilter.PODCASTS),
        )
    }

    @Test
    fun all_chip_reads_the_scope_total_and_missing_types_read_zero() {
        assertEquals(66, counts.countFor(ContentTypeFilter.ALL))
        assertEquals(38, counts.countFor(ContentTypeFilter.ARTICLES))
        assertEquals(0, counts.countFor(ContentTypeFilter.PODCASTS))
    }
}
