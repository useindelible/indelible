package app.indelible.home.model

import app.indelible.api.generated.models.HomeItemResponse
import kotlinx.datetime.Instant
import kotlin.test.Test
import kotlin.test.assertEquals

class HomeProgressTest {
    @Test
    fun backend_percent_converts_to_ui_fraction() {
        assertEquals(0.06f, itemWithProgress(6f).progressFraction)
        assertEquals(0.75f, itemWithProgress(75f).progressFraction)
    }

    @Test
    fun missing_and_out_of_range_progress_is_clamped() {
        assertEquals(0f, itemWithProgress(null).progressFraction)
        assertEquals(0f, itemWithProgress(-5f).progressFraction)
        assertEquals(1f, itemWithProgress(125f).progressFraction)
    }

    private fun itemWithProgress(progressPercent: Float?) =
        HomeItemResponse(
            id = "doc_test",
            itemType = "article",
            title = "Test article",
            createdAt = Instant.parse("2026-07-28T12:00:00Z"),
            progressPercent = progressPercent,
        )
}
