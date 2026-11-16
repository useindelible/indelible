package app.indelible.home.viewmodel

import app.indelible.core.offline.DL_SCOPE
import app.indelible.core.offline.DownloadHarness
import app.indelible.core.offline.OutboxPayload
import app.indelible.core.offline.SessionState
import app.indelible.core.offline.downloadHarness
import app.indelible.core.offline.enqueue
import app.indelible.core.offline.seed
import app.indelible.home.repository.homeRow
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.awaitCancellation
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.launch
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.TestScope
import kotlinx.coroutines.test.advanceTimeBy
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runCurrent
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import kotlin.test.AfterTest
import kotlin.test.BeforeTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

@OptIn(ExperimentalCoroutinesApi::class)
class OfflineShelfViewModelTest {
    private val reachable = MutableStateFlow<Boolean?>(false)
    private val online = MutableStateFlow(true)
    private var probes = 0
    private var drains = 0

    @BeforeTest
    fun setUp() {
        Dispatchers.setMain(StandardTestDispatcher())
    }

    @AfterTest
    fun tearDown() {
        Dispatchers.resetMain()
    }

    private suspend fun TestScope.shelf(
        probe: suspend () -> Unit = { probes++ },
    ): Pair<DownloadHarness, OfflineShelfViewModel> {
        val h = downloadHarness()
        val viewModel =
            OfflineShelfViewModel(
                h.signedIn.registry,
                h.store,
                reachable,
                probe = probe,
                online = online,
                requestDrain = { drains++ },
            )
        backgroundScope.launch { viewModel.state.collect {} }
        return h to viewModel
    }

    @Test
    fun retryNowProbesAndRequestsADrain() =
        runTest {
            reachable.value = true
            val (_, shelf) = shelf()
            runCurrent()

            shelf.retryNow()
            runCurrent()

            assertEquals(1, probes)
            assertEquals(1, drains)
        }

    @Test
    fun retryNowRequestsADrainBeforeTheProbeAnswers() =
        runTest {
            reachable.value = true
            val (_, shelf) = shelf(probe = { awaitCancellation() })
            runCurrent()

            shelf.retryNow()
            runCurrent()

            assertEquals(1, drains)
        }

    @Test
    fun blockedDependantsStayQueuedWhenTheirCreateFails() =
        runTest {
            reachable.value = true
            val (h, shelf) = shelf()
            val create = h.signedIn.enqueue(OutboxPayload.HighlightCreate("hlt_1", "yellow", "quoted", null, null))
            h.signedIn.enqueue(OutboxPayload.HighlightColor("hlt_1", "blue"))
            h.signedIn.enqueue(OutboxPayload.HighlightNote("hlt_1", "a note"))
            h.store.failCreateAndBlockDependants(DL_SCOPE, create, "hlt_1", "bad locator")
            runCurrent()

            assertEquals(1, shelf.progress.value.attention)
            assertEquals(2, shelf.progress.value.queued)
        }

    @Test
    fun attentionCountsFailedChangesTheWayStorageGroupsThem() =
        runTest {
            reachable.value = true
            val (h, shelf) = shelf()
            val failed = h.signedIn.enqueue(OutboxPayload.DocumentNote("mine", null))
            h.signedIn.enqueue(OutboxPayload.DocumentNote("later", null))
            h.store.markFailed(DL_SCOPE, failed, "rejected")
            runCurrent()

            assertEquals(1, shelf.progress.value.attention)
            assertEquals(1, shelf.progress.value.queued)
        }

    @Test
    fun anAnsweringServerOnAPhoneWithoutNetworkIsOfflineNotUnreachable() =
        runTest {
            reachable.value = true
            val (_, shelf) = shelf()
            runCurrent()
            assertEquals(ShelfNet.ONLINE, shelf.progress.value.net)

            online.value = false
            runCurrent()
            assertEquals(ShelfNet.OFFLINE, shelf.progress.value.net)
            assertEquals(false, shelf.progress.value.unreachable)

            online.value = true
            reachable.value = false
            runCurrent()
            assertEquals(true, shelf.progress.value.unreachable)
        }

    @Test
    fun offlineShowsTheCopiesMostRecentlyOpenedFirst() =
        runTest {
            val (h, shelf) = shelf()
            h.seed("doc_old", bytes = 10, openedAt = 1)
            h.seed("doc_new", bytes = 10, openedAt = 2, pinned = true)
            runCurrent()

            val state = shelf.state.value
            assertEquals(ShelfNet.OFFLINE, state.net)
            assertEquals(listOf("doc_new", "doc_old"), state.items.map { it.documentId })
            assertEquals(listOf(true, false), state.items.map { it.pinned })
        }

    @Test
    fun reconnectingSendsTheQueueThenSyncsThenLeaves() =
        runTest {
            val (h, shelf) = shelf()
            val id = h.signedIn.enqueue(OutboxPayload.DocumentNote("mine", null))
            runCurrent()
            assertEquals(1, shelf.state.value.queued)

            reachable.value = true
            runCurrent()
            assertEquals(ShelfNet.SENDING, shelf.state.value.net)
            assertEquals(1, shelf.state.value.sendTotal)

            h.store.remove(DL_SCOPE, id)
            advanceTimeBy(SYNCED_HOLD_MS / 2)
            assertEquals(ShelfNet.SYNCED, shelf.state.value.net)
            assertEquals(1, shelf.state.value.sendTotal)

            advanceTimeBy(SYNCED_HOLD_MS)
            assertEquals(ShelfNet.ONLINE, shelf.state.value.net)
        }

    @Test
    fun startingBeforeAnyAnswerNeverShowsTheShelf() =
        runTest {
            reachable.value = null
            val (h, shelf) = shelf()
            h.signedIn.enqueue(OutboxPayload.DocumentNote("mine", null))
            runCurrent()

            assertEquals(ShelfNet.ONLINE, shelf.state.value.net)
        }

    @Test
    fun losingTheServerWhileSyncedCancelsTheHold() =
        runTest {
            val (h, shelf) = shelf()
            val id = h.signedIn.enqueue(OutboxPayload.DocumentNote("mine", null))
            runCurrent()
            reachable.value = true
            runCurrent()
            h.store.remove(DL_SCOPE, id)
            runCurrent()
            assertEquals(ShelfNet.SYNCED, shelf.state.value.net)

            reachable.value = false
            advanceTimeBy(SYNCED_HOLD_MS * 2)
            assertEquals(ShelfNet.OFFLINE, shelf.state.value.net)

            reachable.value = true
            runCurrent()
            assertEquals(ShelfNet.ONLINE, shelf.state.value.net)
        }

    @Test
    fun reconnectingWithNothingQueuedGoesStraightOnline() =
        runTest {
            val (_, shelf) = shelf()
            runCurrent()

            reachable.value = true
            runCurrent()

            assertEquals(ShelfNet.ONLINE, shelf.state.value.net)
        }

    @Test
    fun aQueueThatStopsMovingHandsHomeBack() =
        runTest {
            val (h, shelf) = shelf()
            h.signedIn.enqueue(OutboxPayload.DocumentNote("mine", null))
            runCurrent()

            reachable.value = true
            advanceTimeBy(SEND_STALL_MS - 1)
            assertEquals(ShelfNet.SENDING, shelf.state.value.net)

            advanceTimeBy(2)
            assertEquals(ShelfNet.ONLINE, shelf.state.value.net)
        }

    @Test
    fun aChangeMadeWhileSendingAddsToTheTotal() =
        runTest {
            val (h, shelf) = shelf()
            val first = h.signedIn.enqueue(OutboxPayload.DocumentNote("one", null))
            h.signedIn.enqueue(OutboxPayload.DocumentNote("two", null))
            runCurrent()
            reachable.value = true
            runCurrent()
            h.store.remove(DL_SCOPE, first)
            runCurrent()
            assertEquals(1, shelf.state.value.sent)

            h.signedIn.enqueue(OutboxPayload.DocumentNote("three", null))
            runCurrent()

            assertEquals(1, shelf.state.value.sent)
            assertEquals(3, shelf.state.value.sendTotal)
        }

    @Test
    fun theShelfUsesTheCopysThumbnailThenItsLeadImage() {
        assertEquals("t.png", shelfImageOf("""{"thumbnail_url":"t.png","lead_image_url":"l.png"}"""))
        assertEquals("l.png", shelfImageOf("""{"lead_image_url":"l.png"}"""))
        assertEquals(null, shelfImageOf("""{"title":"no art"}"""))
        assertEquals(null, shelfImageOf("not json"))
    }

    @Test
    fun anUnansweringServerIsAskedAgainUntilItAnswers() =
        runTest {
            val (_, shelf) = shelf()
            runCurrent()

            advanceTimeBy(PROBE_FIRST_MS + 1)
            assertEquals(1, probes)
            advanceTimeBy(PROBE_EVERY_MS)
            assertEquals(2, probes)

            reachable.value = true
            advanceTimeBy(PROBE_EVERY_MS * 3)
            assertEquals(2, probes)
            assertEquals(ShelfNet.ONLINE, shelf.state.value.net)
        }

    @Test
    fun noProbeWhileTheServerAnswers() =
        runTest {
            reachable.value = true
            shelf()
            advanceTimeBy(PROBE_EVERY_MS * 3)

            assertEquals(0, probes)
        }

    @Test
    fun offlineShowsTheLastLoadedSectionsWithTheShelf() =
        runTest {
            val (h, shelf) = shelf()
            h.seed("doc_b", bytes = 10, openedAt = 1, pinned = true)
            h.store.replaceList(h.session, "home:continue", listOf(homeRow("doc_a", "book"), homeRow("doc_b")), at = 5)
            h.store.replaceList(h.session, "home:recent", listOf(homeRow("doc_c", "article", "example.com")), at = 5)
            runCurrent()

            val state = shelf.state.value
            assertEquals(listOf("doc_a", "doc_b"), state.continueReading.map { it.documentId })
            assertEquals(listOf(false, true), state.continueReading.map { it.onPhone })
            assertEquals(listOf(false, true), state.continueReading.map { it.pinned })
            assertEquals("example.com", state.recentlySaved.single().domain)
            assertEquals(5L, state.loadedAt)
            assertEquals(listOf("doc_b"), state.items.map { it.documentId })
        }

    @Test
    fun sectionsFollowAFreshSnapshot() =
        runTest {
            val (h, shelf) = shelf()
            h.store.replaceList(h.session, "home:recent", listOf(homeRow("doc_a")), at = 1)
            runCurrent()

            h.store.replaceList(h.session, "home:recent", listOf(homeRow("doc_z", progress = 62.5f)), at = 2)
            runCurrent()

            val state = shelf.state.value
            val item = state.recentlySaved.single()
            assertEquals("doc_z", item.documentId)
            assertEquals(62, item.progressPercent)
        }

    @Test
    fun anotherScopesRowsAreNotShown() =
        runTest {
            val (h, shelf) = shelf()
            val own = h.session
            val other = h.signedIn.switchTo("http://a.test|u2")
            h.store.replaceList(other, "home:recent", listOf(homeRow("doc_b")), at = 1)
            h.signedIn.registry.publish(SessionState(own.epoch, own))
            runCurrent()

            val state = shelf.state.value
            assertTrue(state.recentlySaved.isEmpty())
        }
}
