package app.indelible.offline.ui

import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.runComposeUiTest
import app.indelible.offline.viewmodel.Availability
import app.indelible.offline.viewmodel.DocumentOfflineStatus
import app.indelible.offline.viewmodel.SyncBadge
import app.indelible.ui.theme.AppTheme
import kotlin.test.Test

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
            setContent { AppTheme { PendingSyncDot(SyncBadge.Pending(3)) } }

            onNodeWithContentDescription("3 changes waiting to sync").assertExists()
        }
}
