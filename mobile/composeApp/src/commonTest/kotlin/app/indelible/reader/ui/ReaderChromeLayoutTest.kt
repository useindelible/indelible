package app.indelible.reader.ui

import androidx.compose.ui.unit.dp
import app.indelible.reader.model.DataPanel
import app.indelible.reader.playback.PlaybackState
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue

class ReaderChromeLayoutTest {
    @Test
    fun chapterBarSitsAboveTheDockWithoutTheMiniBar() {
        assertEquals(112.dp, chapterBarBottom(immersive = false, miniBarClearance = null))
        assertEquals(56.dp, chapterBarBottom(immersive = true, miniBarClearance = null))
    }

    @Test
    fun chapterBarRisesAboveTheMiniBarWhileItShows() {
        assertEquals(148.dp, chapterBarBottom(immersive = false, miniBarClearance = 140.dp))
    }

    @Test
    fun chapterBarNeverDropsBelowItsRestingPlace() {
        assertEquals(56.dp, chapterBarBottom(immersive = true, miniBarClearance = 20.dp))
    }

    @Test
    fun miniBarIsHiddenWhileTheListenPanelIsOpen() {
        assertFalse(isMiniBarShown(PlaybackState(isPlaying = true), DataPanel.LISTEN))
        assertFalse(isMiniBarShown(PlaybackState(), DataPanel.NONE), "nothing to show before playback starts")
    }

    @Test
    fun miniBarShowsWhilePausedMidway() {
        assertTrue(isMiniBarShown(PlaybackState(isPlaying = false, positionMs = 1), DataPanel.NONE))
    }
}
