package app.indelible.reader.ui

import app.indelible.reader.model.DataPanel
import kotlin.test.Test
import kotlin.test.assertFalse
import kotlin.test.assertTrue

class ReaderImmersiveControllerTest {
    private fun primedState(offset: Float = 600f): ReaderChromeState {
        val state = ReaderChromeState()
        state.onScroll(offset, suppressed = false)
        return state
    }

    @Test
    fun chromeStaysUntilAScrollDown() {
        val state = primedState()
        assertFalse(state.immersive, "the chrome must stay while the reader idles")
    }

    @Test
    fun firstScrollEventOnlyPrimesTheBaseline() {
        val state = ReaderChromeState()
        // A restored reading position arrives as one large offset; it must not
        // read as a downward flick.
        state.onScroll(19000f, suppressed = false)
        assertFalse(state.immersive, "scroll restore must not hide the chrome")
    }

    @Test
    fun scrollingDownRetiresTheChrome() {
        val state = primedState()
        state.onScroll(600f + READER_SCROLL_DELTA_PX, suppressed = false)
        assertTrue(state.immersive, "reading forward must send the furniture away")
    }

    @Test
    fun scrollingDownWhileSuppressedKeepsTheChrome() {
        val state = primedState()
        state.onScroll(600f + READER_SCROLL_DELTA_PX * 4, suppressed = true)
        assertFalse(state.immersive, "an open sheet must hold the chrome")
    }

    @Test
    fun scrollingUpRevealsTheChromeLikeATap() {
        val state = primedState()
        state.retire()
        assertTrue(state.immersive)

        state.onScroll(600f - READER_SCROLL_DELTA_PX, suppressed = false)

        assertFalse(state.immersive, "turning back up must call the chrome")
    }

    @Test
    fun jitterBelowTheThresholdChangesNothing() {
        val state = primedState()
        state.retire()

        state.onScroll(600f - (READER_SCROLL_DELTA_PX - 1f), suppressed = false)
        assertTrue(state.immersive, "a thumb settling must not count as scrolling up")

        state.reveal()
        state.onScroll(600f - (READER_SCROLL_DELTA_PX - 1f) + (READER_SCROLL_DELTA_PX - 1f), suppressed = false)
        assertFalse(state.immersive, "small downward jitter must not hide the chrome")
    }

    @Test
    fun revealClearsImmersive() {
        val state = primedState()
        state.retire()

        state.reveal()

        assertFalse(state.immersive)
    }

    @Test
    fun anyOpenSurfaceSuppressesTheChrome() {
        assertFalse(isChromeSuppressed(DataPanel.NONE, false, false, false), "plain reading")
        assertTrue(isChromeSuppressed(DataPanel.LISTEN, false, false, false), "open panel")
        assertTrue(isChromeSuppressed(DataPanel.NONE, true, false, false), "live selection")
        assertTrue(isChromeSuppressed(DataPanel.NONE, false, true, false), "tapped highlight")
        assertTrue(isChromeSuppressed(DataPanel.NONE, false, false, true), "tag sheet")
    }

    @Test
    fun retireIsIdempotent() {
        val state = ReaderChromeState()
        state.retire()
        state.retire()
        assertTrue(state.immersive)
    }
}
