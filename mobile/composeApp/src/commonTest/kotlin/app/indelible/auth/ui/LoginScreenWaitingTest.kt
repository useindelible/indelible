package app.indelible.auth.ui

import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.runComposeUiTest
import app.indelible.ui.theme.AppTheme
import kotlin.test.Test

@OptIn(ExperimentalTestApi::class)
class LoginScreenWaitingTest {
    @Test
    fun theNoticeShowsOnlyWithAQueue() =
        runComposeUiTest {
            setContent { AppTheme { WaitingChangesNotice(waitingChanges = 0) } }
            onNodeWithText("Sign in to finish syncing", substring = true).assertDoesNotExist()
        }

    @Test
    fun theNoticeCountsTheQueue() =
        runComposeUiTest {
            setContent { AppTheme { WaitingChangesNotice(waitingChanges = 3) } }
            onNodeWithText("Sign in to finish syncing · 3 changes waiting").assertExists()
        }
}
