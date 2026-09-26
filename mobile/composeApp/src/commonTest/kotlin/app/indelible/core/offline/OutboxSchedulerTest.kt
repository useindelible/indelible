package app.indelible.core.offline

import kotlinx.coroutines.launch
import kotlinx.coroutines.test.advanceTimeBy
import kotlinx.coroutines.test.runCurrent
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue

class OutboxSchedulerTest {
    private val scope = "scope"

    @Test
    fun requestDuringAPassRunsASecondPass() =
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
            store.enqueueNote(session, "doc_2")
            worker.requestDrain()
            sender.release()
            runCurrent()

            assertEquals(2, sender.calls.size)
            assertTrue(store.pendingOrdered(scope).isEmpty())
        }

    @Test
    fun slowFailureUsesTheClockAtTheOutcomeAndArmsATimerThatFires() =
        runTest {
            val signedIn = signedInTestStore()
            val store = signedIn.store
            val session = signedIn.session
            store.enqueueNote(session, "doc_1")
            val sender = GatedSender()
            sender.enqueueOutcome(SendOutcome.Http(500, null, "boom"))
            val started = sender.holdNextSend()
            val worker = startedWorker(store, sender, signedIn.registry)

            worker.requestDrain()
            started.await()
            advanceTimeBy(10_000)
            sender.release()
            runCurrent()

            val row = store.pendingOrdered(scope).single()
            assertEquals(10_000L + backoffMs(1), row.nextAttemptAt)
            assertEquals(1, sender.calls.size)

            advanceTimeBy(backoffMs(1))
            runCurrent()

            assertEquals(2, sender.calls.size)
            assertTrue(store.pendingOrdered(scope).isEmpty())
        }

    @Test
    fun enqueueAfterAnEmptyPassThenA500ArmsTheRetryTimerFromTheWorkerAlone() =
        runTest {
            val signedIn = signedInTestStore()
            val store = signedIn.store
            val session = signedIn.session
            val sender = GatedSender()
            val worker = startedWorker(store, sender, signedIn.registry)
            runCurrent()
            assertTrue(sender.calls.isEmpty())

            store.enqueueNote(session, "doc_1")
            sender.enqueueOutcome(SendOutcome.Http(500, null, "boom"))
            worker.requestDrain()
            runCurrent()
            assertEquals(1, sender.calls.size)

            advanceTimeBy(backoffMs(1) - 1)
            runCurrent()
            assertEquals(1, sender.calls.size)

            advanceTimeBy(1)
            runCurrent()
            assertEquals(2, sender.calls.size)
        }

    @Test
    fun shieldedDueDependantDoesNotSpinAndTheTimerTargetsTheHeadDeadline() =
        runTest {
            val signedIn = signedInTestStore()
            val store = signedIn.store
            val session = signedIn.session
            val sender = GatedSender()
            val worker = startedWorker(store, sender, signedIn.registry)
            val createId = store.enqueueHighlightCreate(session, "hlt_1", "doc_1")
            store.markAttempt(scope, createId, now = 0, nextAttemptAt = 30_000, error = "500")
            store.enqueueHighlightColor(session, "hlt_1", "doc_1")

            worker.requestDrain()
            advanceTimeBy(29_999)
            runCurrent()
            assertTrue(sender.calls.isEmpty())

            advanceTimeBy(1)
            runCurrent()

            assertEquals(2, sender.calls.size)
            assertTrue(store.pendingOrdered(scope).isEmpty())
        }

    @Test
    fun terminalFailureOnAnEntityLeavesItsLaterDueRowsToAFollowUpPass() =
        runTest {
            val signedIn = signedInTestStore()
            val store = signedIn.store
            val session = signedIn.session
            val sender = GatedSender()
            sender.enqueueOutcome(SendOutcome.Http(422, retryAfterSeconds = null, message = "unprocessable"))
            val worker = startedWorker(store, sender, signedIn.registry)
            store.enqueueHighlightColor(session, "hlt_1", "doc_1")
            store.enqueueHighlightColor(session, "hlt_1", "doc_1")

            worker.requestDrain()
            runCurrent()

            assertEquals(2, sender.calls.size)
            assertTrue(store.pendingOrdered(scope).isEmpty())
            assertEquals(1, store.rowsByState(scope, OutboxState.FAILED).size)
        }

    @Test
    fun completedPassWithNothingWaitingArmsNoTimer() =
        runTest {
            val signedIn = signedInTestStore()
            val store = signedIn.store
            val session = signedIn.session
            store.enqueueNote(session, "doc_1")
            val sender = GatedSender()
            val worker = startedWorker(store, sender, signedIn.registry)

            worker.requestDrain()
            runCurrent()
            advanceTimeBy(HOURLY)
            runCurrent()

            assertEquals(1, sender.calls.size)
        }

    @Test
    fun authPauseAndTransitionPauseArmNothingAndClearingOneDoesNotClearTheOther() =
        runTest {
            val signedIn = signedInTestStore()
            val store = signedIn.store
            val session = signedIn.session
            store.enqueueNote(session, "doc_1")
            val sender = GatedSender()
            sender.enqueueOutcome(SendOutcome.Http(401, null, "expired"))
            val worker = startedWorker(store, sender, signedIn.registry)
            worker.requestDrain()
            runCurrent()
            assertTrue(worker.authPaused.value)
            advanceTimeBy(HOURLY)
            runCurrent()
            assertEquals(1, sender.calls.size)

            worker.beginTransition()
            worker.resumeAuth()
            runCurrent()
            assertEquals(1, sender.calls.size)
            assertFalse(worker.authPaused.value)

            worker.endTransition()
            runCurrent()
            assertEquals(2, sender.calls.size)
        }

    @Test
    fun noSendBeforeStartupReadyAndTheRequestIsServedOnceReady() =
        runTest {
            val signedIn = signedInTestStore()
            val store = signedIn.store
            val session = signedIn.session
            store.enqueueNote(session, "doc_1")
            val sender = GatedSender()
            val worker = testWorker(store, sender, signedIn.registry)
            worker.start()

            worker.requestDrain()
            runCurrent()
            assertTrue(sender.calls.isEmpty())

            worker.markStartupReady()
            runCurrent()
            assertEquals(1, sender.calls.size)
        }

    @Test
    fun beginTransitionAwaitsTheRunningPass() =
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

            var returned = false
            val transition =
                launch {
                    worker.beginTransition()
                    returned = true
                }
            runCurrent()
            assertFalse(returned)

            sender.release()
            transition.join()
            assertTrue(returned)
            worker.endTransition()
        }

    @Test
    fun requestWithNoScopeSendsNothingAndArmsNoTimer() =
        runTest {
            val store = testStore()
            val sender = GatedSender()
            val worker = startedWorker(store, sender, SessionRegistry())

            worker.requestDrain()
            runCurrent()
            advanceTimeBy(HOURLY)
            runCurrent()

            assertTrue(store.scopesWithState().isEmpty())
            assertTrue(sender.calls.isEmpty())
        }

    private companion object {
        const val HOURLY = 3_600_000L
    }
}
