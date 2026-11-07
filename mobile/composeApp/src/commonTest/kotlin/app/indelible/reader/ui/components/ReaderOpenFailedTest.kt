package app.indelible.reader.ui.components

import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.runComposeUiTest
import app.indelible.ui.theme.AppTheme
import kotlin.test.Test
import kotlin.test.assertEquals

@OptIn(ExperimentalTestApi::class)
class ReaderOpenFailedTest {
    @Test
    fun anItemThatCouldNotOpenOffersRetryAndAWayBack() =
        runComposeUiTest {
            val taps = mutableListOf<String>()
            setContent {
                AppTheme {
                    ReaderOpenFailed("Couldn't load", onRetry = { taps += "retry" }, onBack = { taps += "back" })
                }
            }

            onNodeWithText("Retry").performClick()
            onNodeWithText("Back").performClick()

            assertEquals(listOf("retry", "back"), taps)
        }
}
