package app.indelible.reader.ui.components

import androidx.compose.ui.semantics.ProgressBarRangeInfo
import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.hasProgressBarRangeInfo
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.runComposeUiTest
import app.indelible.reader.model.PdfStatus
import app.indelible.reader.model.ReaderBackground
import app.indelible.ui.theme.AppTheme
import app.indelible.ui.theme.ReaderBgDark
import app.indelible.ui.theme.ReaderBgPaper
import app.indelible.ui.theme.ReaderBgSepia
import app.indelible.ui.theme.ReaderBgSlate
import app.indelible.ui.theme.ReaderCanvasColors
import app.indelible.ui.theme.canvasFor
import kotlin.test.Test
import kotlin.test.assertEquals

@OptIn(ExperimentalTestApi::class)
class PdfStatusContentTest {
    @Test
    fun waitingShowsTheDownloadProgress() =
        runComposeUiTest {
            setContent { AppTheme { PdfStatusContent(PdfStatus.WAITING_FOR_COPY, downloadFraction = 0.4f, {}, {}) } }

            onNodeWithText("Getting this PDF").assertIsDisplayed()
            onNode(hasProgressBarRangeInfo(ProgressBarRangeInfo(0.4f, 0f..1f))).assertIsDisplayed()
        }

    @Test
    fun waitingWithUnknownSizeShowsAnIndeterminateBar() =
        runComposeUiTest {
            setContent { AppTheme { PdfStatusContent(PdfStatus.WAITING_FOR_COPY, downloadFraction = null, {}, {}) } }

            onNode(hasProgressBarRangeInfo(ProgressBarRangeInfo.Indeterminate)).assertIsDisplayed()
        }

    @Test
    fun connectionFailureOffersRetry() =
        runComposeUiTest {
            var retried = false
            setContent {
                AppTheme { PdfStatusContent(PdfStatus.CONNECTION_FAILED, null, onRetry = { retried = true }, {}) }
            }

            onNodeWithText("Couldn’t reach your server").assertIsDisplayed()
            onNodeWithText("Retry").performClick()

            assertEquals(true, retried)
        }

    @Test
    fun noSpaceOffersStorageAndRetry() =
        runComposeUiTest {
            var opened = false
            setContent {
                AppTheme {
                    PdfStatusContent(PdfStatus.NO_SPACE, null, onRetry = {}, onManageStorage = { opened = true })
                }
            }

            onNodeWithText("Retry").assertIsDisplayed()
            onNodeWithText("Manage storage").performClick()

            assertEquals(true, opened)
        }

    @Test
    fun downloadFailureOffersRetry() =
        runComposeUiTest {
            setContent { AppTheme { PdfStatusContent(PdfStatus.DOWNLOAD_FAILED, null, {}, {}) } }

            onNodeWithText("Couldn’t get this PDF").assertIsDisplayed()
            onNodeWithText("Retry").assertIsDisplayed()
        }

    @Test
    fun unreadableOffersNoAction() =
        runComposeUiTest {
            setContent { AppTheme { PdfStatusContent(PdfStatus.UNREADABLE, null, {}, {}) } }

            onNodeWithText("Couldn’t open this PDF").assertIsDisplayed()
            onNodeWithText("Retry").assertDoesNotExist()
            onNodeWithText("Manage storage").assertDoesNotExist()
        }

    @Test
    fun canvasFollowsTheReaderBackground() {
        val canvas = ReaderCanvasColors(ReaderBgPaper, ReaderBgSepia, ReaderBgSlate, ReaderBgDark)

        assertEquals(ReaderBgPaper, canvas.canvasFor(ReaderBackground.PAPER))
        assertEquals(ReaderBgSepia, canvas.canvasFor(ReaderBackground.SEPIA))
        assertEquals(ReaderBgSlate, canvas.canvasFor(ReaderBackground.SLATE))
        assertEquals(ReaderBgDark, canvas.canvasFor(ReaderBackground.BLACK))
    }
}
