package app.indelible.core.offline

import kotlinx.coroutines.test.advanceTimeBy
import kotlinx.coroutines.test.runCurrent
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class OutboxWorkerRecoveryTest {
    private val scope = "scope"

    @Test
    fun failureReadingPendingRowsLeavesRowsPendingAndTheRecoveryTimerRunsThem() =
        runTest {
            val signedIn = signedInTestStore()
            val backing = signedIn.store
            val session = signedIn.session
            backing.enqueueNote(session, "doc_1")
            val sender = GatedSender()
            startedWorker(ThrowingPendingReadStore(backing), sender, signedIn.registry)

            assertTrue(sender.calls.isEmpty())
            assertEquals(1, backing.pendingOrdered(scope).size)

            advanceTimeBy(backoffMs(1))
            runCurrent()

            assertEquals(1, sender.calls.size)
            assertTrue(backing.pendingOrdered(scope).isEmpty())
        }

    @Test
    fun failureMarkingAnAttemptLeavesTheRowUntouchedAndRecovers() =
        runTest {
            val signedIn = signedInTestStore()
            val backing = signedIn.store
            val session = signedIn.session
            backing.enqueueNote(session, "doc_1")
            val sender = GatedSender()
            sender.enqueueOutcome(SendOutcome.Http(500, null, "boom"))
            startedWorker(ThrowingMarkAttemptStore(backing), sender, signedIn.registry)

            assertEquals(1, sender.calls.size)
            assertEquals(0, backing.pendingOrdered(scope).single().attempts)

            advanceTimeBy(backoffMs(1))
            runCurrent()

            assertEquals(2, sender.calls.size)
            assertTrue(backing.pendingOrdered(scope).isEmpty())
        }

    @Test
    fun failureReadingClientIdentitySendsNothingAndRecovers() =
        runTest {
            val signedIn = signedInTestStore()
            val backing = signedIn.store
            val session = signedIn.session
            backing.enqueueNote(session, "doc_1")
            val sender = GatedSender()
            startedWorker(ThrowingClientIdentityStore(backing), sender, signedIn.registry)

            assertTrue(sender.calls.isEmpty())
            assertEquals(1, backing.pendingOrdered(scope).size)

            advanceTimeBy(backoffMs(1))
            runCurrent()

            assertEquals(1, sender.calls.size)
        }

    @Test
    fun consecutiveFailuresBackOffFurther() =
        runTest {
            val signedIn = signedInTestStore()
            val backing = signedIn.store
            val session = signedIn.session
            backing.enqueueNote(session, "doc_1")
            val store = ThrowingClientIdentityStore(ThrowingClientIdentityStore(backing))
            val sender = GatedSender()
            startedWorker(store, sender, signedIn.registry)

            advanceTimeBy(backoffMs(1))
            runCurrent()
            advanceTimeBy(backoffMs(2) - 1)
            runCurrent()
            assertTrue(sender.calls.isEmpty())

            advanceTimeBy(1)
            runCurrent()

            assertEquals(1, sender.calls.size)
        }

    @Test
    fun stopCancelsTheRunningPassAndNoLaterRequestIsServed() =
        runTest {
            val signedIn = signedInTestStore()
            val store = signedIn.store
            val session = signedIn.session
            store.enqueueNote(session, "doc_1")
            val sender = GatedSender()
            val started = sender.holdNextSend()
            val worker = startedWorker(store, sender, signedIn.registry)
            worker.requestDrain()
            started.await()

            worker.stop()
            sender.release()
            runCurrent()
            assertEquals(1, sender.calls.size)
            assertEquals(1, store.pendingOrdered(scope).size)

            worker.requestDrain()
            runCurrent()
            assertEquals(1, sender.calls.size)
        }
}
