package app.indelible.profile.ui

import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.hasClickAction
import androidx.compose.ui.test.hasText
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.runComposeUiTest
import app.indelible.ui.theme.AppTheme
import kotlin.test.Test
import kotlin.test.assertTrue

@OptIn(ExperimentalTestApi::class)
class SignOutDialogTest {
    @Test
    fun withNothingWaitingItOnlyAsksToConfirm() =
        runComposeUiTest {
            var confirmed = false
            setContent {
                AppTheme { SignOutDialog(waitingChanges = 0, onConfirm = { confirmed = true }, onDismiss = {}) }
            }

            onNodeWithText("Are you sure you want to sign out?").assertIsDisplayed()
            onNodeWithText("not reached the server", substring = true).assertDoesNotExist()
            onNode(hasText("Sign Out") and hasClickAction()).performClick()

            assertTrue(confirmed)
        }

    @Test
    fun withChangesWaitingItSaysTheyAreLost() =
        runComposeUiTest {
            var dismissed = false
            setContent {
                AppTheme { SignOutDialog(waitingChanges = 3, onConfirm = {}, onDismiss = { dismissed = true }) }
            }

            onNodeWithText("3 changes have not reached the server yet. Signing out deletes them from this phone.")
                .assertIsDisplayed()
            onNodeWithText("Are you sure you want to sign out?").assertDoesNotExist()
            onNodeWithText("Cancel").performClick()

            assertTrue(dismissed)
        }
}
