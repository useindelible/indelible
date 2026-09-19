package app.indelible.offline.viewmodel

import app.indelible.core.offline.DL_DOC
import app.indelible.core.offline.DL_SCOPE
import app.indelible.core.offline.DownloadHarness
import app.indelible.core.offline.OutboxPayload
import app.indelible.core.offline.downloadHarness
import app.indelible.core.offline.enqueue
import app.indelible.core.offline.seed
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
import kotlin.test.assertFalse
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
        return DocumentOfflineViewModel(documentId, h.signedIn.registry, statuses, h.manager).also { viewModel ->
            backgroundScope.launch { viewModel.status.collect {} }
        }
    }

    @Test
    fun keepOfflinePinsAndDownloadsWhenAbsent() =
        runTest {
            val h = downloadHarness()
            h.seed("doc_a", bytes = 10)
            val present = offline(h, "doc_a")
            val absent = offline(h)

            present.setKeepOffline(true)
            absent.setKeepOffline(true)
            advanceUntilIdle()

            assertEquals(Availability.KEPT, present.status.value.availability)
            assertEquals(Availability.KEPT, absent.status.value.availability)
            assertEquals(1L, h.store.cachedDocument(DL_SCOPE, DL_DOC)?.generation)
        }

    @Test
    fun turningOffUnpinsOnly() =
        runTest {
            val h = downloadHarness()
            h.seed(DL_DOC, bytes = 10, pinned = true)
            val offline = offline(h)

            offline.setKeepOffline(false)
            advanceUntilIdle()

            assertEquals(Availability.CACHED, offline.status.value.availability)
            assertFalse(checkNotNull(h.store.cachedDocument(DL_SCOPE, DL_DOC)).pinned)
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

            offline.removeFromDevice()
            advanceUntilIdle()

            assertNull(h.store.cachedDocument(DL_SCOPE, DL_DOC))
            assertEquals(Availability.NOT_ON_DEVICE, offline.status.value.availability)
            assertEquals(SyncBadge.Pending(1), offline.status.value.sync)
        }
}
