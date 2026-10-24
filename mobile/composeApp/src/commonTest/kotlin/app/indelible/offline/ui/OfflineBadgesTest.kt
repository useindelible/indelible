package app.indelible.offline.ui

import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.runComposeUiTest
import app.indelible.offline.viewmodel.Availability
import app.indelible.offline.viewmodel.DocumentOfflineStatus
import app.indelible.offline.viewmodel.SyncBadge
import app.indelible.ui.theme.AppTheme
import kotlin.test.Test
import kotlin.test.assertTrue

@OptIn(ExperimentalTestApi::class)
class OfflineBadgesTest {
    @Test
    fun badgesNameAvailabilityAndSync() =
        runComposeUiTest {
            setContent {
                AppTheme { OfflineBadges(DocumentOfflineStatus(Availability.CACHED, SyncBadge.Failed(1))) }
            }

            onNodeWithText("Cached").assertIsDisplayed()
            onNodeWithText("Sync failed").assertIsDisplayed()
        }

    @Test
    fun pendingDotDescribesWaitingChanges() =
        runComposeUiTest {
            setContent { AppTheme { PendingSyncDot(SyncBadge.Pending(3), onClick = {}) } }

            onNodeWithContentDescription("3 changes waiting to sync").assertExists()
        }

    @Test
    fun pendingDotOpensThePendingList() =
        runComposeUiTest {
            var opened = false
            setContent { AppTheme { PendingSyncDot(SyncBadge.Pending(1), onClick = { opened = true }) } }

            onNodeWithContentDescription("1 change waiting to sync").performClick()

            assertTrue(opened)
        }
}
