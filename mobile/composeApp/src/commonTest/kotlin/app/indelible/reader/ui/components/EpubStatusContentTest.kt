package app.indelible.reader.ui.components

import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.runComposeUiTest
import app.indelible.reader.viewmodel.EpubStatus
import app.indelible.ui.theme.AppTheme
import kotlin.test.Test
import kotlin.test.assertEquals

@OptIn(ExperimentalTestApi::class)
class EpubStatusContentTest {
    @Test
    fun unavailableChapterOffersRetry() =
        runComposeUiTest {
            var retried = false
            setContent {
                AppTheme {
                    EpubStatusContent(EpubStatus.CHAPTER_UNAVAILABLE, onRetry = { retried = true }, onReprocess = {})
                }
            }

            onNodeWithText("Retry").performClick()

            assertEquals(true, retried)
        }

    @Test
    fun oldServerAsksForAnUpdate() =
        runComposeUiTest {
            setContent { AppTheme { EpubStatusContent(EpubStatus.SERVER_TOO_OLD, onRetry = {}, onReprocess = {}) } }

            onNodeWithText("Update your server").assertIsDisplayed()
        }

    @Test
    fun connectionFailureOffersRetryNotAnUpdate() =
        runComposeUiTest {
            setContent { AppTheme { EpubStatusContent(EpubStatus.CONNECTION_FAILED, onRetry = {}, onReprocess = {}) } }

            onNodeWithText("Retry").assertIsDisplayed()
            onNodeWithText("Update your server").assertDoesNotExist()
        }

    @Test
    fun needsReprocessOffersReprocess() =
        runComposeUiTest {
            var reprocessed = false
            setContent {
                AppTheme {
                    EpubStatusContent(EpubStatus.NEEDS_REPROCESS, onRetry = {}, onReprocess = { reprocessed = true })
                }
            }

            onNodeWithText("Process again").performClick()

            assertEquals(true, reprocessed)
        }
}
