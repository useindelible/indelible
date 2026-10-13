package app.indelible.home.viewmodel

import app.indelible.core.offline.DL_SCOPE
import app.indelible.core.offline.DownloadHarness
import app.indelible.core.offline.OutboxPayload
import app.indelible.core.offline.downloadHarness
import app.indelible.core.offline.enqueue
import app.indelible.core.offline.seed
import kotlinx.coroutines.Dispatchers
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

@OptIn(ExperimentalCoroutinesApi::class)
class OfflineShelfViewModelTest {
    private val reachable = MutableStateFlow<Boolean?>(false)
    private var probes = 0

    @BeforeTest
    fun setUp() {
        Dispatchers.setMain(StandardTestDispatcher())
    }

    @AfterTest
    fun tearDown() {
        Dispatchers.resetMain()
    }

    private suspend fun TestScope.shelf(): Pair<DownloadHarness, OfflineShelfViewModel> {
        val h = downloadHarness()
        val viewModel = OfflineShelfViewModel(h.signedIn.registry, h.store, reachable, probe = { probes++ })
        backgroundScope.launch { viewModel.state.collect {} }
        return h to viewModel
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
}
