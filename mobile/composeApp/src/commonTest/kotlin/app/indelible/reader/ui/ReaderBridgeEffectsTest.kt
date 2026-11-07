package app.indelible.reader.ui

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.runComposeUiTest
import app.indelible.reader.model.ReaderPreferences
import kotlin.test.Test
import kotlin.test.assertContains
import kotlin.test.assertEquals
import kotlin.test.assertTrue

@OptIn(ExperimentalTestApi::class)
class ReaderBridgeEffectsTest {
    @Test
    fun aClearRequestDropsThePageSelection() =
        runComposeUiTest {
            val calls = mutableListOf<String>()
            var clear by mutableStateOf<Long?>(null)
            setContent { ReaderBridgeEffects(inputs(clearSelection = clear), {}, {}) { calls += it } }
            waitForIdle()
            assertTrue(calls.none { "clearSelection" in it }, "calls $calls")

            clear = 1L
            waitForIdle()

            assertEquals(1, calls.count { it == "window.clearSelection();" })
        }

    @Test
    fun thePageCanDropItsOwnSelection() {
        assertContains(READER_BRIDGE_JS, "window.clearSelection = function")
        assertContains(READER_BRIDGE_JS, "removeAllRanges()")
    }

    private fun inputs(clearSelection: Long?) =
        ReaderBridgeInputs(
            pageLoaded = true,
            scrollToPercent = null,
            anchorScroll = null,
            preferences = ReaderPreferences(),
            highlights = emptyList(),
            speakingSentenceIndex = -1,
            immersive = false,
            scrollToCharOffset = null,
            captureRequest = null,
            clearSelection = clearSelection,
        )
}
