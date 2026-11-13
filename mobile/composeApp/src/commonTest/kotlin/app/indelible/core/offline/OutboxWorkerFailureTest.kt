package app.indelible.core.offline

import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.test.runCurrent
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNull
import kotlin.test.assertTrue

private const val THEIRS_MS = 2_000L
private const val THIRD_MS = 3_000L
private val TOO_OLD = SendOutcome.Http(405, null, "method not allowed")
private val REJECTED = SendOutcome.Http(400, null, "bad request", ApiProblem(detail = "Note too long"))
private val UNPREPARED =
    SendOutcome.Http(422, null, "unprocessable", ApiProblem(errors = listOf(FieldError("document_id", "not ready"))))

@OptIn(ExperimentalCoroutinesApi::class)
class OutboxWorkerFailureTest {
    private val scope = "scope"

    @Test
    fun row17StopsThatKindForThePassAndLeavesOtherKindsSending() =
        runTest {
            val signedIn = signedInTestStore()
            val store = signedIn.store
            val session = signedIn.session
            val sender = FakeOutboxSender()
            val worker = startedWorker(store, sender, signedIn.registry)
            val firstEvent = store.enqueueReadingEvent(session, "doc_1", recordedAtEpochMs = 1L)
            val secondEvent = store.enqueueReadingEvent(session, "doc_2", recordedAtEpochMs = 2L)
            store.enqueueNote(session, "doc_3")
            sender.enqueueOutcome(TOO_OLD)
            sender.enqueueOutcome(SendOutcome.Success())

            worker.requestDrain()
            runCurrent()

            val sentKinds = sender.calls.map { it.single().kind }
            assertEquals(listOf(OutboxKind.READING_EVENT, OutboxKind.DOCUMENT_NOTE), sentKinds)
            val failed = store.rowsByState(scope, OutboxState.FAILED).single()
            assertEquals(firstEvent, failed.id)
            assertEquals(SyncFailure.Kind.SERVER_TOO_OLD, failed.failure?.kind)
            assertEquals(SyncFailure.Feature.READING_EVENTS, failed.failure?.feature)
            assertEquals(listOf(secondEvent), store.pendingOrdered(scope).map { it.id })
        }

    @Test
    fun anUnansweredCapabilityHoldsNotesAndLeavesOtherKindsSending() =
        runTest {
            val signedIn = signedInTestStore()
            val store = signedIn.store
            val session = signedIn.session
            val sender = FakeOutboxSender()
            val worker = startedWorker(store, sender, signedIn.registry)
            val note = store.enqueueNote(session, "doc_1")
            store.enqueueNote(session, "doc_2")
            store.enqueueReadingEvent(session, "doc_3", recordedAtEpochMs = 1L)
            sender.enqueueOutcome(SendOutcome.NoAnswer)
            sender.enqueueOutcome(SendOutcome.Success())

            worker.requestDrain()
            runCurrent()

            val sentKinds = sender.calls.map { it.single().kind }
            assertEquals(listOf(OutboxKind.DOCUMENT_NOTE, OutboxKind.READING_EVENT), sentKinds)
            val waiting = store.pendingOrdered(scope)
            assertEquals(2, waiting.size)
            assertEquals(1, waiting.single { it.id == note }.attempts)
        }

    @Test
    fun anOldServerHoldsOnlyTheCheckedNote() =
        runTest {
            val signedIn = signedInTestStore()
            val sender = FakeOutboxSender()
            val worker = startedWorker(signedIn.store, sender, signedIn.registry)
            val checked = signedIn.store.enqueueNote(signedIn.session, "doc_1")
            signedIn.store.enqueueNote(signedIn.session, "doc_2")
            sender.enqueueOutcome(SendOutcome.Unsupported)
            sender.enqueueOutcome(SendOutcome.Success())

            worker.requestDrain()
            runCurrent()

            assertEquals(2, sender.calls.size)
            val failed = signedIn.store.rowsByState(scope, OutboxState.FAILED).single()
            assertEquals(checked, failed.id)
            assertEquals(SyncFailure.Kind.SERVER_TOO_OLD, failed.failure?.kind)
            assertEquals(SyncFailure.Feature.NOTES, failed.failure?.feature)
        }

    @Test
    fun keepMineMeetsAThirdEditAsANewConflict() =
        runTest {
            val signedIn = signedInTestStore()
            val sender = FakeOutboxSender()
            val worker = startedWorker(signedIn.store, sender, signedIn.registry)
            val note = signedIn.store.enqueueNote(signedIn.session, "doc_1")
            sender.enqueueOutcome(SendOutcome.NoteConflict("theirs", THEIRS_MS))
            sender.enqueueOutcome(SendOutcome.NoteConflict("third", THIRD_MS))
            worker.requestDrain()
            runCurrent()

            assertTrue(signedIn.store.keepLocalNote(signedIn.session, note, THEIRS_MS))
            worker.requestDrain()
            runCurrent()

            val failed = signedIn.store.rowsByState(scope, OutboxState.FAILED).single()
            assertEquals(THIRD_MS, failed.failure?.serverNoteUpdatedAtEpochMs)
            assertFalse(signedIn.store.keepLocalNote(signedIn.session, note, THEIRS_MS))
        }

    @Test
    fun aFailedRowPersistsItsFailureAndRetryClearsIt() =
        runTest {
            val signedIn = signedInTestStore()
            val store = signedIn.store
            val sender = FakeOutboxSender()
            val worker = startedWorker(store, sender, signedIn.registry)
            val id = store.enqueueNote(signedIn.session, "doc_1")
            sender.enqueueOutcome(REJECTED)

            worker.requestDrain()
            runCurrent()

            val failed = store.rowsByState(scope, OutboxState.FAILED).single()
            val expected = SyncFailure(SyncFailure.Kind.REJECTED, status = 400, serverMessage = "Note too long")
            assertEquals(expected, failed.failure)
            assertEquals("Note too long", failed.lastError)

            assertTrue(store.retryRow(scope, id))
            val pending = store.pendingOrdered(scope).single()
            assertNull(pending.failure)
            assertNull(pending.lastError)
        }

    @Test
    fun serverTooOldRowsAreRevivedOnceAtColdStart() =
        runTest {
            val signedIn = signedInTestStore()
            val store = signedIn.store
            val session = signedIn.session
            val tooOld = store.enqueueNote(session, "doc_1")
            val rejected = store.enqueueNote(session, "doc_2")
            store.markFailed(scope, tooOld, SyncFailure(SyncFailure.Kind.SERVER_TOO_OLD, status = 405))
            store.markFailed(scope, rejected, rejected("no"))
            val sender = FakeOutboxSender()
            sender.enqueueOutcome(SendOutcome.Success())

            startedWorker(store, sender, signedIn.registry)
            runCurrent()

            assertEquals(listOf(tooOld), sender.calls.map { it.single().id })
            assertEquals(listOf(rejected), store.rowsByState(scope, OutboxState.FAILED).map { it.id })
        }

    @Test
    fun anUnpreparedItemIsRecordedAsStillPreparing() =
        runTest {
            val signedIn = signedInTestStore()
            val store = signedIn.store
            val sender = FakeOutboxSender()
            val worker = startedWorker(store, sender, signedIn.registry)
            val id = store.enqueueHighlightCreate(signedIn.session, "hlt_1", "doc_1")
            sender.enqueueOutcome(UNPREPARED)

            worker.requestDrain()
            runCurrent()

            val row = store.pendingOrdered(scope).single()
            assertEquals(id, row.id)
            assertEquals(STILL_PREPARING_ERROR, row.lastError)
            assertEquals(HOURLY_CAP_MS, row.nextAttemptAt)
        }
}
