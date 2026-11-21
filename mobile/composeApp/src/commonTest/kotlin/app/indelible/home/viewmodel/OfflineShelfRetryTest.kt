package app.indelible.home.viewmodel

import app.indelible.core.offline.DL_SCOPE
import app.indelible.core.offline.DownloadHarness
import app.indelible.core.offline.OutboxPayload
import app.indelible.core.offline.downloadHarness
import app.indelible.core.offline.enqueue
import app.indelible.core.offline.seed
import kotlinx.coroutines.CompletableDeferred
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
import kotlin.test.assertTrue

private const val FAR_FUTURE = 4_102_444_800_000L

@OptIn(ExperimentalCoroutinesApi::class)
class OfflineShelfRetryTest {
    private val reachable = MutableStateFlow<Boolean?>(false)
    private val online = MutableStateFlow(true)
    private var probes = 0
    private var drains = 0
    private var probeGate: CompletableDeferred<Unit>? = null

    @BeforeTest
    fun setUp() {
        Dispatchers.setMain(StandardTestDispatcher())
    }

    @AfterTest
    fun tearDown() {
        Dispatchers.resetMain()
    }

    private suspend fun TestScope.shelf(watchItems: Boolean = true): Pair<DownloadHarness, OfflineShelfViewModel> {
        val h = downloadHarness()
        val viewModel =
            OfflineShelfViewModel(
                h.signedIn.registry,
                h.store,
                reachable,
                probe = {
                    probes++
                    probeGate?.await()
                },
                online = online,
                requestDrain = { drains++ },
            )
        backgroundScope.launch { if (watchItems) viewModel.state.collect {} else viewModel.progress.collect {} }
        return h to viewModel
    }

    @Test
    fun retryNowClearsTheBackoffOfQueuedRowsBeforeDraining() =
        runTest {
            reachable.value = true
            val (h, shelf) = shelf()
            val id = h.signedIn.enqueue(OutboxPayload.DocumentNote("mine", null))
            h.store.markAttempt(DL_SCOPE, id, now = 1_000L, nextAttemptAt = FAR_FUTURE, error = "timeout")
            runCurrent()

            shelf.retryNow()
            runCurrent()

            val row = h.store.pendingOrdered(DL_SCOPE).single()
            assertEquals(0L, row.nextAttemptAt)
            assertEquals(1, drains)
        }

    @Test
    fun retryNowWhileAProbeIsInFlightDoesNotProbeAgain() =
        runTest {
            reachable.value = true
            probeGate = CompletableDeferred()
            val (_, shelf) = shelf()
            runCurrent()

            shelf.retryNow()
            shelf.retryNow()
            runCurrent()

            assertEquals(1, probes)
            probeGate?.complete(Unit)
        }

    @Test
    fun theNetworkReturningAsksTheServerAgainAtOnce() =
        runTest {
            online.value = false
            val (_, shelf) = shelf()
            runCurrent()
            advanceTimeBy(PROBE_EVERY_MS)
            runCurrent()
            assertEquals(0, probes)

            online.value = true
            runCurrent()
            assertTrue(shelf.progress.value.unreachable)
            advanceTimeBy(PROBE_FIRST_MS)
            runCurrent()

            assertEquals(1, probes)
            reachable.value = true
        }

    @Test
    fun hostKeepsANonDefaultPort() {
        assertEquals("10.0.2.2:38473", hostOf("http://10.0.2.2:38473"))
        assertEquals("indelible.home.arpa", hostOf("https://indelible.home.arpa"))
        assertEquals("indelible.home.arpa", hostOf("https://indelible.home.arpa:443/"))
    }

    @Test
    fun theBannerAloneDoesNotBuildTheShelf() =
        runTest {
            reachable.value = true
            val (h, shelf) = shelf(watchItems = false)
            h.seed("doc_1", bytes = 10, openedAt = 1)
            runCurrent()

            assertTrue(shelf.state.value.items.isEmpty())

            val watch = backgroundScope.launch { shelf.state.collect {} }
            runCurrent()

            val items = shelf.state.value.items
            assertEquals(listOf("doc_1"), items.map { it.documentId })
            watch.cancel()
        }
}
