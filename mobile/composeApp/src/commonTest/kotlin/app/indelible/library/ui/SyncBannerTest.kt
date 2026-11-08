package app.indelible.library.ui

import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.runComposeUiTest
import app.indelible.home.viewmodel.ShelfNet
import app.indelible.home.viewmodel.SyncProgress
import app.indelible.ui.theme.AppTheme
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull
import kotlin.test.assertTrue

@OptIn(ExperimentalTestApi::class)
class SyncBannerTest {
    @Test
    fun unreachableOutranksEverythingElse() {
        val progress = SyncProgress(net = ShelfNet.OFFLINE, queued = 4, attention = 1, unreachable = true)

        assertEquals(SyncBanner.Unreachable("indelible.home.arpa", 4), syncBannerOf(progress, "indelible.home.arpa"))
        assertNull(syncBannerOf(progress, host = null))
    }

    @Test
    fun sendingShowsNOfM() {
        val progress = SyncProgress(net = ShelfNet.SENDING, queued = 2, sendTotal = 3)

        assertEquals(SyncBanner.Syncing(sent = 1, total = 3), syncBannerOf(progress, "h"))
    }

    @Test
    fun attentionAfterARunShowsSyncedAndReview() {
        val progress = SyncProgress(net = ShelfNet.ONLINE, attention = 1, lastSent = 2)

        assertEquals(SyncBanner.Attention(synced = 2, needAttention = 1), syncBannerOf(progress, "h"))
    }

    @Test
    fun syncedWithoutAttentionIsAllSynced() {
        assertEquals(SyncBanner.AllSynced, syncBannerOf(SyncProgress(net = ShelfNet.SYNCED, sendTotal = 3), "h"))
    }

    @Test
    fun offlineAndQuietOnlineShowNoBanner() {
        assertNull(syncBannerOf(SyncProgress(net = ShelfNet.OFFLINE, queued = 2, attention = 1), "h"))
        assertNull(syncBannerOf(SyncProgress(net = ShelfNet.ONLINE), "h"))
    }

    @Test
    fun theSyncingBannerNamesTheCount() =
        runComposeUiTest {
            setContent {
                AppTheme { SyncBannerBar(SyncBanner.Syncing(sent = 1, total = 3), onReview = {}, onRetryNow = {}) }
            }

            onNodeWithText("Back online · syncing 2 of 3").assertIsDisplayed()
            onNodeWithText("1 sent · 2 to go").assertIsDisplayed()
        }

    @Test
    fun reviewNavigates() =
        runComposeUiTest {
            var reviewed = false
            setContent {
                AppTheme { SyncBannerBar(SyncBanner.Attention(2, 1), onReview = { reviewed = true }, onRetryNow = {}) }
            }

            onNodeWithText("2 synced · 1 needs attention").assertIsDisplayed()
            onNodeWithText("Review").performClick()

            assertTrue(reviewed)
        }

    @Test
    fun retryNowCallsBack() =
        runComposeUiTest {
            var retried = false
            setContent {
                AppTheme {
                    SyncBannerBar(
                        SyncBanner.Unreachable("indelible.home.arpa", 4),
                        onReview = {},
                        onRetryNow = { retried = true },
                    )
                }
            }

            onNodeWithText("Can’t reach indelible.home.arpa").assertIsDisplayed()
            onNodeWithText("Retry now").performClick()

            assertTrue(retried)
        }

    @Test
    fun anEmptyQueueIsNotCounted() =
        runComposeUiTest {
            setContent {
                AppTheme { SyncBannerBar(SyncBanner.Unreachable("h", 0), onReview = {}, onRetryNow = {}) }
            }

            onNodeWithText("This phone is online. The server is not answering, so nothing has been lost.")
                .assertIsDisplayed()
        }
}
