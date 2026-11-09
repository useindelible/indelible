package app.indelible.offline.viewmodel

import app.indelible.core.offline.DL_DOC
import app.indelible.core.offline.DL_SCOPE
import app.indelible.core.offline.OutboxPayload
import app.indelible.core.offline.OutboxState
import app.indelible.core.offline.STILL_PREPARING_ERROR
import app.indelible.core.offline.SessionState
import app.indelible.core.offline.SyncFailure
import app.indelible.core.offline.enqueue
import app.indelible.core.offline.enqueueProgress
import app.indelible.core.offline.rejected
import app.indelible.core.offline.row
import app.indelible.core.offline.seed
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.test.StandardTestDispatcher
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

private const val THEIRS_AT = 2_000L

private fun conflict() =
    SyncFailure(SyncFailure.Kind.NOTE_CONFLICT, serverNoteBody = "theirs", serverNoteUpdatedAtEpochMs = THEIRS_AT)

private val TOO_OLD =
    SyncFailure(SyncFailure.Kind.SERVER_TOO_OLD, status = 405, feature = SyncFailure.Feature.READING_EVENTS)
private val DELETED = SyncFailure(SyncFailure.Kind.DELETED_ELSEWHERE, status = 404, serverCode = "not_found")

@OptIn(ExperimentalCoroutinesApi::class)
class StorageFailureViewModelTest {
    @BeforeTest
    fun setUp() {
        Dispatchers.setMain(StandardTestDispatcher())
    }

    @AfterTest
    fun tearDown() {
        Dispatchers.resetMain()
    }

    private suspend fun Storage.conflictedNote(): String {
        h.seed(DL_DOC, bytes = 10)
        val id = h.signedIn.enqueue(OutboxPayload.DocumentNote("mine", 1_000L))
        h.store.markFailed(DL_SCOPE, id, conflict())
        return id
    }

    @Test
    fun discardRemovesTheChangeAndItsBlockedDependants() =
        runTest {
            val s = storage()
            val create = s.h.signedIn.enqueue(OutboxPayload.HighlightCreate("hlt_9", "yellow", "quoted", null, null))
            s.h.signedIn.enqueue(OutboxPayload.HighlightColor("hlt_9", "blue"))
            s.h.store.failCreateAndBlockDependants(DL_SCOPE, create, "hlt_9", rejected("rejected"))
            advanceUntilIdle()
            assertTrue(s.change(create).canDiscard)

            s.viewModel.discard(create)
            advanceUntilIdle()

            assertTrue(s.state.changes.isEmpty())
            assertTrue(s.h.store.observeOutbox(DL_SCOPE).first().isEmpty())
        }

    @Test
    fun keepMineRebasesAndRequestsADrain() =
        runTest {
            val s = storage()
            val id = s.conflictedNote()
            advanceUntilIdle()
            assertFalse(s.change(id).canRetry)
            assertFalse(s.change(id).canDiscard)

            s.viewModel.keepMine(id, THEIRS_AT)
            advanceUntilIdle()

            val row = s.h.signedIn.row(id)
            assertEquals(OutboxState.PENDING, row.state)
            assertEquals(OutboxPayload.DocumentNote("mine", THEIRS_AT), row.payload)
            assertEquals(1, s.drains)
            assertEquals(listOf(StorageNotice.KEPT_MINE), s.notices())
        }

    @Test
    fun keepOtherRestoresTheServerNoteAndDropsTheRow() =
        runTest {
            val s = storage()
            val id = s.conflictedNote()
            advanceUntilIdle()

            s.viewModel.keepOther(id, THEIRS_AT)
            advanceUntilIdle()

            assertTrue(s.state.changes.isEmpty())
            val copy = checkNotNull(s.h.store.cachedDocument(DL_SCOPE, DL_DOC))
            assertEquals("theirs", copy.noteBody)
            assertEquals(THEIRS_AT, copy.noteServerUpdatedAt)
            assertEquals(0, s.drains)
            assertEquals(listOf(StorageNotice.KEPT_OTHER), s.notices())
        }

    @Test
    fun aRefusedResolutionShowsTheRefreshedNotice() =
        runTest {
            val s = storage()
            val id = s.h.signedIn.enqueue(OutboxPayload.DocumentNote("mine", null))
            s.h.store.markFailed(DL_SCOPE, id, rejected("Note too long"))
            advanceUntilIdle()

            s.viewModel.keepMine(id, THEIRS_AT)
            advanceUntilIdle()

            assertEquals(listOf(StorageNotice.CHANGED_REFRESHED), s.notices())
            assertEquals(ChangeStatus.FAILED, s.change(id).status)
            assertEquals(0, s.drains)
        }

    @Test
    fun aServerTooOldChangeCannotBeRetried() =
        runTest {
            val s = storage()
            val id = s.h.signedIn.enqueueProgress(5_000)
            s.h.store.markFailed(DL_SCOPE, id, TOO_OLD)
            advanceUntilIdle()

            val change = s.change(id)
            assertEquals(SyncFailure.Kind.SERVER_TOO_OLD, change.failure?.kind)
            assertFalse(change.canRetry)
            assertFalse(change.canDiscard)

            s.viewModel.retry(id)
            advanceUntilIdle()
            assertEquals(0, s.drains)
        }

    @Test
    fun aDeletedItemsChangeOffersItsTextAndDiscardOnly() =
        runTest {
            val s = storage()
            val id = s.h.signedIn.enqueue(OutboxPayload.HighlightCreate("hlt_9", "yellow", "quoted", null, null))
            s.h.store.failCreateAndBlockDependants(DL_SCOPE, id, "hlt_9", DELETED)
            advanceUntilIdle()

            val change = s.change(id)
            assertEquals("quoted", change.copyText)
            assertTrue(change.canDiscard)
            assertFalse(change.canRetry)
        }

    @Test
    fun anUnpreparedItemShowsStillPreparing() =
        runTest {
            val s = storage()
            val preparing = s.h.signedIn.enqueue(OutboxPayload.HighlightCreate("hlt_9", "yellow", "quoted", null, null))
            s.h.store.markAttempt(DL_SCOPE, preparing, now = 1, nextAttemptAt = 2, error = STILL_PREPARING_ERROR)
            val retrying = s.h.signedIn.enqueue(OutboxPayload.DocumentNote("draft", null))
            s.h.store.markAttempt(DL_SCOPE, retrying, now = 1, nextAttemptAt = 2, error = "timeout")
            advanceUntilIdle()

            assertTrue(s.change(preparing).stillPreparing)
            assertEquals(ChangeStatus.RETRYING, s.change(preparing).status)
            assertFalse(s.change(retrying).stillPreparing)
        }

    @Test
    fun onlineWithoutAnAnswerMarksTheQueueUnreachable() =
        runTest {
            val online = MutableStateFlow(true)
            val reachable = MutableStateFlow<Boolean?>(null)
            val s = storage(online = online, reachable = reachable)
            advanceUntilIdle()
            assertFalse(s.state.serverUnreachable)

            reachable.value = false
            advanceUntilIdle()
            assertTrue(s.state.serverUnreachable)

            online.value = false
            advanceUntilIdle()
            assertFalse(s.state.serverUnreachable)
        }

    @Test
    fun retryNowRequestsADrain() =
        runTest {
            val s = storage()
            val id = s.h.signedIn.enqueue(OutboxPayload.DocumentNote("draft", null))
            s.h.store.markAttempt(DL_SCOPE, id, now = 1, nextAttemptAt = 999_999, error = "timeout")
            advanceUntilIdle()

            s.viewModel.retryNow()
            advanceUntilIdle()

            assertEquals(1, s.drains)
            assertEquals(0L, s.h.signedIn.row(id).nextAttemptAt)
        }

    @Test
    fun aResolutionAgainstAnOlderSheetIsRefused() =
        runTest {
            val s = storage()
            val id = s.conflictedNote()
            advanceUntilIdle()

            s.viewModel.keepOther(id, seenServerUpdatedAt = 1_500L)
            advanceUntilIdle()

            assertEquals(listOf(StorageNotice.CHANGED_REFRESHED), s.notices())
            assertEquals(ChangeStatus.FAILED, s.change(id).status)
        }

    @Test
    fun discardIsRefusedWhenTheChangeCannotBeDiscarded() =
        runTest {
            val s = storage()
            val id = s.h.signedIn.enqueueProgress(5_000)
            s.h.store.markFailed(DL_SCOPE, id, TOO_OLD)
            advanceUntilIdle()

            s.viewModel.discard(id)
            advanceUntilIdle()

            assertEquals(ChangeStatus.FAILED, s.change(id).status)
        }

    @Test
    fun aSignOutBetweenTapAndWriteIsSwallowed() =
        runTest {
            val s = storage()
            val id = s.conflictedNote()
            advanceUntilIdle()
            val registry = s.h.signedIn.registry

            s.viewModel.keepMine(id, THEIRS_AT)
            s.viewModel.discard(id)
            registry.publish(SessionState(registry.current.value.epoch, session = null))
            advanceUntilIdle()

            assertEquals(OutboxState.FAILED, s.h.signedIn.row(id).state)
            assertEquals(0, s.drains)
        }
}
