package app.indelible.offline.viewmodel

import app.indelible.core.offline.Acquisition
import app.indelible.core.offline.CAPABILITIES
import app.indelible.core.offline.CHAPTER_LIST_SUPPORTED
import app.indelible.core.offline.DL_DOC
import app.indelible.core.offline.DL_SCOPE
import app.indelible.core.offline.DownloadFailure
import app.indelible.core.offline.DownloadHarness
import app.indelible.core.offline.FakeServer
import app.indelible.core.offline.OutboxPayload
import app.indelible.core.offline.downloadHarness
import app.indelible.core.offline.enqueue
import app.indelible.core.offline.json
import app.indelible.core.offline.seed
import io.ktor.client.engine.mock.respond
import io.ktor.http.HttpStatusCode
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.launch
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.TestScope
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import kotlin.test.AfterTest
import kotlin.test.BeforeTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull

@OptIn(ExperimentalCoroutinesApi::class)
class DocumentOfflineViewModelTest {
    @BeforeTest
    fun setUp() {
        Dispatchers.setMain(StandardTestDispatcher())
    }

    @AfterTest
    fun tearDown() {
        Dispatchers.resetMain()
    }

    private fun TestScope.offline(
        h: DownloadHarness,
        documentId: String = DL_DOC,
    ): DocumentOfflineViewModel {
        val statuses = OfflineStatuses(h.signedIn.registry, h.store, h.manager.acquisitions)
        return DocumentOfflineViewModel(documentId, statuses).also { viewModel ->
            backgroundScope.launch { viewModel.status.collect {} }
        }
    }

    @Test
    fun statusFollowsTheSignedInScope() =
        runTest {
            val h = downloadHarness()
            h.seed(DL_DOC, bytes = 10, pinned = true)
            val offline = offline(h)
            advanceUntilIdle()
            assertEquals(Availability.KEPT, offline.status.value.availability)

            h.signedIn.switchTo("http://a.test|u2")
            advanceUntilIdle()

            assertEquals(DocumentOfflineStatus(), offline.status.value)
        }

    @Test
    fun removeDeletesCopy() =
        runTest {
            val h = downloadHarness()
            h.seed(DL_DOC, bytes = 10, pinned = true)
            h.signedIn.enqueue(OutboxPayload.DocumentNote("mine", null))
            val offline = offline(h)
            advanceUntilIdle()
            assertEquals(SyncBadge.Pending(1), offline.status.value.sync)

            h.manager.removeFromDevice(DL_SCOPE, DL_DOC)
            advanceUntilIdle()

            assertNull(h.store.cachedDocument(DL_SCOPE, DL_DOC))
            assertEquals(Availability.NOT_ON_DEVICE, offline.status.value.availability)
            assertEquals(SyncBadge.Pending(1), offline.status.value.sync)
        }

    @Test
    fun removingACopyLeavesTheSheetShowingNothingOnTheDevice() =
        runTest {
            val h = downloadHarness()
            h.seed(DL_DOC, bytes = 6_500_000, pinned = true)
            val offline = offline(h)
            advanceUntilIdle()
            assertEquals(Availability.KEPT, offline.status.value.availability)

            h.manager.removeFromDevice(DL_SCOPE, DL_DOC)
            advanceUntilIdle()

            assertEquals(Availability.NOT_ON_DEVICE, offline.status.value.availability)
        }

    @Test
    fun removingDuringADownloadLeavesTheSheetShowingNothingOnTheDevice() =
        runTest {
            val h = downloadHarness()
            val offline = offline(h)
            h.manager.keepOffline(h.session, DL_DOC)
            advanceUntilIdle()
            assertEquals(Availability.KEPT, offline.status.value.availability)

            h.manager.removeFromDevice(DL_SCOPE, DL_DOC)
            advanceUntilIdle()

            assertEquals(Availability.NOT_ON_DEVICE, offline.status.value.availability)
        }

    @Test
    fun aRemovedCopyStaysGoneWhileTheSheetIsOpen() =
        runTest {
            val h = downloadHarness()
            val offline = offline(h)
            h.manager.keepOffline(h.session, DL_DOC)
            advanceUntilIdle()

            h.manager.removeFromDevice(DL_SCOPE, DL_DOC)
            advanceUntilIdle()
            advanceUntilIdle()

            assertNull(h.store.cachedDocument(DL_SCOPE, DL_DOC))
            assertEquals(Availability.NOT_ON_DEVICE, offline.status.value.availability)
        }

    @Test
    fun aFailedKeepOnAnOldServerFailsOnTheBoard() =
        runTest {
            val h = book { route(CAPABILITIES) { json("""{"capabilities":[]}""") } }
            val offline = offline(h)

            h.manager.keepOffline(h.session, DL_DOC)
            advanceUntilIdle()

            assertEquals(Acquisition.Failed(DownloadFailure.SERVER_TOO_OLD), h.acquisition())
        }

    @Test
    fun aFailedKeepWithoutConnectionIsBlankThenKept() =
        runTest {
            val h = book { route(CAPABILITIES) { respond("", HttpStatusCode.ServiceUnavailable) } }
            val offline = offline(h)
            h.manager.keepOffline(h.session, DL_DOC)
            advanceUntilIdle()
            assertEquals(Acquisition.Failed(DownloadFailure.NETWORK), h.acquisition())
            assertEquals(Availability.NOT_ON_DEVICE, offline.status.value.availability)

            h.server.route(CAPABILITIES) { json(CHAPTER_LIST_SUPPORTED) }
            h.manager.keepOffline(h.session, DL_DOC)
            advanceUntilIdle()

            assertEquals(Availability.KEPT, offline.status.value.availability)
        }

    private suspend fun TestScope.book(configure: FakeServer.() -> Unit) =
        downloadHarness {
            documentType = "book"
            available = listOf("epub")
            configure()
        }
}
