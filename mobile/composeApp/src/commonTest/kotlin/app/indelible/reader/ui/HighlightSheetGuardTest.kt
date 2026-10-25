package app.indelible.reader.ui

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.runComposeUiTest
import app.indelible.reader.model.HighlightData
import app.indelible.reader.viewmodel.FakeReaderRepository.Companion.fakeHighlight
import kotlin.test.Test
import kotlin.test.assertEquals

@OptIn(ExperimentalTestApi::class)
class HighlightSheetGuardTest {
    @Test
    fun sheetClosesWhenItsHighlightLeavesTheState() =
        runComposeUiTest {
            var highlight by mutableStateOf<HighlightData?>(fakeHighlight())
            var closed = 0
            setContent {
                CloseSheetWhenHighlightGone("hlt_test1", highlight, highlightsLoaded = true) { closed++ }
            }
            waitForIdle()
            assertEquals(0, closed)

            highlight = null
            waitForIdle()

            assertEquals(1, closed)
        }

    @Test
    fun sheetWaitsWhileTheHighlightsAreStillLoading() =
        runComposeUiTest {
            var loaded by mutableStateOf(false)
            var closed = 0
            setContent { CloseSheetWhenHighlightGone("hlt_test1", null, highlightsLoaded = loaded) { closed++ } }
            waitForIdle()
            assertEquals(0, closed)

            loaded = true
            waitForIdle()

            assertEquals(1, closed)
        }

    @Test
    fun noSheetMeansNothingToClose() =
        runComposeUiTest {
            var closed = 0
            setContent { CloseSheetWhenHighlightGone(null, null, highlightsLoaded = true, onClose = { closed++ }) }
            waitForIdle()

            assertEquals(0, closed)
        }
}
