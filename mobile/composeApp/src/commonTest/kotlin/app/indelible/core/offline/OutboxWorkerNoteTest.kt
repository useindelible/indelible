package app.indelible.core.offline

import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.test.runCurrent
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

@OptIn(ExperimentalCoroutinesApi::class)
class OutboxWorkerNoteTest {
    @Test
    fun aSentNoteStampsTheCopyWithTheServersTimestamp() =
        runTest {
            val signedIn = signedInTestStore()
            val store = signedIn.store
            signedIn.installNow(serverDocument(note = "server"))
            val sender = FakeOutboxSender()
            val worker = startedWorker(store, sender, signedIn.registry)
            signedIn.enqueue(OutboxPayload.DocumentNote("mine", 1_000L))
            sender.enqueueOutcome(SendOutcome.Success(noteUpdatedAtEpochMs = 2_000L))

            worker.requestDrain()
            runCurrent()

            assertEquals(2_000L, store.cachedDocuments(signedIn.scope).single().noteServerUpdatedAt)
            assertTrue(store.pendingOrdered(signedIn.scope).isEmpty())
        }

    @Test
    fun aSecondEditInTheSameWalkIsSentOverTheFirstOnesStamp() =
        runTest {
            val signedIn = signedInTestStore()
            val sender = FakeOutboxSender()
            val worker = startedWorker(signedIn.store, sender, signedIn.registry)
            signedIn.enqueue(OutboxPayload.DocumentNote("first", 1_000L))
            signedIn.enqueue(OutboxPayload.DocumentNote("second", 1_000L))
            sender.enqueueOutcome(SendOutcome.Success(noteUpdatedAtEpochMs = 2_000L))
            sender.enqueueOutcome(SendOutcome.Success(noteUpdatedAtEpochMs = 3_000L))

            worker.requestDrain()
            runCurrent()

            assertEquals(OutboxPayload.DocumentNote("second", 2_000L), sender.calls[1].single().payload)
            assertTrue(signedIn.store.pendingOrdered(signedIn.scope).isEmpty())
        }

    @Test
    fun aNoteConflictFailsTheRowWithTheServersNote() =
        runTest {
            val signedIn = signedInTestStore()
            val store = signedIn.store
            val sender = FakeOutboxSender()
            val worker = startedWorker(store, sender, signedIn.registry)
            signedIn.enqueue(OutboxPayload.DocumentNote("mine", 1_000L))
            sender.enqueueOutcome(SendOutcome.NoteConflict("theirs", 2_000L))

            worker.requestDrain()
            runCurrent()

            val failed = store.rowsByState(signedIn.scope, OutboxState.FAILED).single()
            val expected =
                SyncFailure(
                    SyncFailure.Kind.NOTE_CONFLICT,
                    serverNoteBody = "theirs",
                    serverNoteUpdatedAtEpochMs = 2_000L,
                )
            assertEquals(expected, failed.failure)
            assertTrue(store.pendingOrdered(signedIn.scope).isEmpty())
        }
}
