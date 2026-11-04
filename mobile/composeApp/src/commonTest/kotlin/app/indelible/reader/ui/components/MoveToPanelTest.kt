package app.indelible.reader.ui.components

import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.runComposeUiTest
import app.indelible.ui.theme.AppTheme
import kotlin.test.Test

@OptIn(ExperimentalTestApi::class)
class MoveToPanelTest {
    @Test
    fun theCurrentBucketIsMarked() =
        runComposeUiTest {
            setContent { AppTheme { MoveToPanel(currentState = "later", onMove = {}) } }

            onNodeWithContentDescription("Current location").assertIsDisplayed()
        }

    @Test
    fun anUnknownBucketMarksNoRow() =
        runComposeUiTest {
            setContent { AppTheme { MoveToPanel(currentState = null, onMove = {}) } }

            onNodeWithContentDescription("Current location").assertDoesNotExist()
        }
}
