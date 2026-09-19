package app.indelible.offline.viewmodel

import app.indelible.core.offline.DL_DOC
import app.indelible.core.offline.DL_SCOPE
import app.indelible.core.offline.DownloadHarness
import app.indelible.core.offline.OfflineCopies
import app.indelible.core.offline.OutboxPayload
import app.indelible.core.offline.OutboxState
import app.indelible.core.offline.downloadHarness
import app.indelible.core.offline.enqueue
import app.indelible.core.offline.enqueueProgress
import app.indelible.core.offline.row
import app.indelible.core.offline.seed
import app.indelible.core.storage.DEFAULT_OFFLINE_CAP_BYTES
import app.indelible.core.storage.InMemoryUserPreferencesStorage
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.first
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
import kotlin.test.assertTrue

private class Storage(
    val h: DownloadHarness,
    val preferences: InMemoryUserPreferencesStorage,
) {
    var drains = 0
    val viewModel =
        StorageViewModel(
            registry = h.signedIn.registry,
            store = h.store,
            copies = OfflineCopies(h.store, h.files) { preferences.getOfflineCapBytes() },
            downloads = h.manager,
            preferences = preferences,
            requestDrain = { drains++ },
        )

    val state: StorageUiState get() = viewModel.state.value

    fun change(id: String): PendingChange = state.changes.single { it.id == id }
}

@OptIn(ExperimentalCoroutinesApi::class)
class StorageViewModelTest {
    @BeforeTest
    fun setUp() {
        Dispatchers.setMain(StandardTestDispatcher())
    }

    @AfterTest
    fun tearDown() {
        Dispatchers.resetMain()
    }

    private suspend fun TestScope.storage(cap: Long = DEFAULT_OFFLINE_CAP_BYTES): Storage {
        val preferences = InMemoryUserPreferencesStorage().apply { saveOfflineCapBytes(cap) }
        return Storage(downloadHarness(), preferences).also { storage ->
            backgroundScope.launch { storage.viewModel.state.collect {} }
        }
    }

    @Test
    fun capChangePersistsAndEnforces() =
        runTest {
            val s = storage()
            s.h.seed("doc_a", bytes = 100, openedAt = 1)
            s.h.seed("doc_b", bytes = 100, openedAt = 2)

            s.viewModel.setCap(150)
            advanceUntilIdle()

            assertEquals(150L, s.preferences.getOfflineCapBytes())
            assertEquals(150L, s.state.capBytes)
            assertEquals(listOf("doc_b"), s.state.cached.map { it.documentId })
        }

    @Test
    fun retryResetsFailedRowAndRequestsDrain() =
        runTest {
            val s = storage()
            val id = s.h.signedIn.enqueue(OutboxPayload.DocumentNote("mine", null))
            s.h.store.markFailed(DL_SCOPE, id, "rejected")

            s.viewModel.retry(id)
            advanceUntilIdle()

            assertEquals(OutboxState.PENDING, s.h.signedIn.row(id).state)
            assertEquals(1, s.drains)
            assertEquals(ChangeStatus.PENDING, s.change(id).status)
        }

    @Test
    fun removeDownloadsLeavesOutboxAndReportsQueuedCounts() =
        runTest {
            val s = storage()
            s.h.seed("doc_a", bytes = 100)
            s.h.seed("doc_b", bytes = 100, pinned = true)
            s.h.signedIn.enqueue(OutboxPayload.DocumentNote("mine", null))
            s.h.signedIn.enqueueProgress(5_000)
            advanceUntilIdle()
            assertEquals(2, s.state.queuedChanges)

            s.viewModel.removeDownloads()
            advanceUntilIdle()

            assertEquals(emptyList(), s.state.kept + s.state.cached)
            assertEquals(0L, s.state.usedBytes)
            assertEquals(2, s.state.queuedChanges)
            assertEquals(2, s.h.store.observeOutbox(DL_SCOPE).first().size)
        }

    @Test
    fun removeCopyDeletesOnlyThatDocument() =
        runTest {
            val s = storage()
            s.h.seed("doc_a", bytes = 100, pinned = true)
            s.h.seed("doc_b", bytes = 100)

            s.viewModel.removeCopy("doc_a")
            advanceUntilIdle()

            assertEquals(emptyList(), s.state.kept)
            assertEquals(listOf("doc_b"), s.state.cached.map { it.documentId })
        }

    @Test
    fun overCapByKeptSaysManageKept() =
        runTest {
            val s = storage(cap = 150)
            s.h.seed("doc_a", bytes = 100, pinned = true)
            s.h.seed("doc_b", bytes = 100)
            advanceUntilIdle()
            assertFalse(s.state.keptOverCap)

            s.h.store.setPinned(DL_SCOPE, "doc_b", pinned = true)
            advanceUntilIdle()

            assertTrue(s.state.keptOverCap)
        }

    @Test
    fun failedRowsOfferRetryOnly() =
        runTest {
            val s = storage()
            s.h.seed(DL_DOC, bytes = 10)
            val waiting = s.h.signedIn.enqueueProgress(5_000)
            val retrying = s.h.signedIn.enqueueProgress(6_000)
            s.h.store.markAttempt(DL_SCOPE, retrying, now = 1, nextAttemptAt = 2, error = "timeout")
            val failed = s.h.signedIn.enqueue(OutboxPayload.DocumentNote("mine", null))
            s.h.store.markFailed(DL_SCOPE, failed, "Note too long")
            advanceUntilIdle()

            assertEquals(ChangeStatus.PENDING, s.change(waiting).status)
            assertEquals(ChangeStatus.RETRYING, s.change(retrying).status)
            assertEquals(ChangeStatus.FAILED, s.change(failed).status)
            assertEquals("Note too long", s.change(failed).error)
            assertEquals("Title", s.change(failed).documentTitle)
            assertEquals(listOf(failed), s.state.changes.filter { it.canRetry }.map { it.id })
        }

    @Test
    fun supersededFailedRowOffersNoRetry() =
        runTest {
            val s = storage()
            val older = s.h.signedIn.enqueue(OutboxPayload.DocumentNote("older", null))
            s.h.store.markFailed(DL_SCOPE, older, "rejected")
            val newer = s.h.signedIn.enqueue(OutboxPayload.DocumentNote("newer", null))
            s.h.store.remove(DL_SCOPE, newer)
            advanceUntilIdle()

            assertEquals(ChangeStatus.SUPERSEDED, s.change(older).status)
            assertFalse(s.change(older).canRetry)
            assertEquals(0, s.state.queuedChanges)

            s.viewModel.retry(older)
            advanceUntilIdle()

            assertEquals(0, s.drains)
            assertEquals(OutboxState.FAILED, s.h.signedIn.row(older).state)
        }

    @Test
    fun blockedChangesSitUnderTheirFailedCreate() =
        runTest {
            val s = storage()
            val create = s.h.signedIn.enqueue(OutboxPayload.HighlightCreate("hlt_9", "yellow", "quoted", null, null))
            val colour = s.h.signedIn.enqueue(OutboxPayload.HighlightColor("hlt_9", "blue"))
            s.h.store.failCreateAndBlockDependants(DL_SCOPE, create, "hlt_9", "rejected")
            advanceUntilIdle()

            val change = s.state.changes.single()
            assertEquals(create, change.id)
            assertEquals(listOf(colour), change.blocked.map { it.id })
            assertEquals(ChangeStatus.BLOCKED, change.blocked.single().status)
            assertEquals(2, s.state.queuedChanges)
        }

    @Test
    fun showsOnlyTheSignedInScope() =
        runTest {
            val s = storage()
            s.h.seed("doc_a", bytes = 100)
            s.h.signedIn.enqueue(OutboxPayload.DocumentNote("mine", null))
            advanceUntilIdle()
            assertEquals(1, s.state.cached.size)
            assertEquals(1, s.state.changes.size)

            s.h.signedIn.switchTo("http://a.test|u2")
            advanceUntilIdle()

            assertEquals(emptyList(), s.state.cached)
            assertEquals(emptyList(), s.state.changes)
        }
}
