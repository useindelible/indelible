package app.indelible.reader.ui.components

import app.indelible.reader.viewmodel.TocStatus
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue

class ContentsPanelStateTest {
    @Test
    fun dots_classify_done_here_and_upcoming_around_the_active_index() {
        assertEquals(TocDotState.DONE, dotState(index = 0, activeIndex = 1))
        assertEquals(TocDotState.HERE, dotState(index = 1, activeIndex = 1))
        assertEquals(TocDotState.UPCOMING, dotState(index = 2, activeIndex = 1))
        // Before any section is entered nothing reads as done.
        assertEquals(TocDotState.UPCOMING, dotState(index = 0, activeIndex = -1))
    }

    @Test
    fun eyebrow_carries_progress_only_when_ready() {
        assertEquals("Contents / 21% read", contentsEyebrow(TocStatus.READY, 21))
        assertEquals("Contents", contentsEyebrow(TocStatus.PENDING, 21))
        assertEquals("Contents", contentsEyebrow(TocStatus.NONE, 0))
    }

    @Test
    fun contents_pill_shows_only_for_a_ready_outline() {
        assertTrue(showContentsPill(TocStatus.READY))
        assertFalse(showContentsPill(TocStatus.LOADING))
        assertFalse(showContentsPill(TocStatus.PENDING))
        assertFalse(showContentsPill(TocStatus.NONE))
        assertFalse(showContentsPill(TocStatus.UNAVAILABLE))
    }
}
