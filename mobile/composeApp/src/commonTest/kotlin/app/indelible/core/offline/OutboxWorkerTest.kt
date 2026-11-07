package app.indelible.core.offline

import kotlinx.coroutines.test.runCurrent
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue

class OutboxWorkerTest {
    private val scope = "scope"

    @Test
    fun http401PausesAuthAndResumeAuthAllowsTheRowToDrainAgain() =
        runTest {
            val signedIn = signedInTestStore()
            val store = signedIn.store
            val session = signedIn.session
            val sender = FakeOutboxSender()
            val worker = startedWorker(store, sender, signedIn.registry)
            val rowId = store.enqueueNote(session, "doc_1")
            val followingId = store.enqueueNote(session, "doc_2")
            sender.enqueueOutcome(SendOutcome.Http(401, null, "expired"))

            worker.requestDrain()
            runCurrent()

            assertTrue(worker.authPaused.value)
            assertEquals(1, sender.calls.size)
            val pendingRows = store.rowsByState(scope, OutboxState.PENDING)
            val pending = pendingRows.single { it.id == rowId }
            assertEquals(0, pending.attempts)
            val untouched = pendingRows.single { it.id == followingId }
            assertEquals(0, untouched.attempts)

            worker.requestDrain()
            runCurrent()
            assertEquals(1, sender.calls.size)

            sender.enqueueOutcome(SendOutcome.Success)
            sender.enqueueOutcome(SendOutcome.Success)
            worker.resumeAuth()
            runCurrent()
            assertFalse(worker.authPaused.value)

            assertTrue(store.pendingOrdered(scope).isEmpty())
        }

    @Test
    fun terminalCreateShieldsSiblingForThePassAndBlocksItInTheStore() =
        runTest {
            val signedIn = signedInTestStore()
            val store = signedIn.store
            val session = signedIn.session
            val sender = FakeOutboxSender()
            val worker = startedWorker(store, sender, signedIn.registry)
            val createId = store.enqueueHighlightCreate(session, "hlt_1", "doc_1")
            val colorId = store.enqueueHighlightColor(session, "hlt_1", "doc_1")
            val otherEntityId = store.enqueueNote(session, "doc_2")
            sender.enqueueOutcome(SendOutcome.Http(422, null, "anchor invalid"))
            sender.enqueueOutcome(SendOutcome.Success)

            worker.requestDrain()
            runCurrent()

            assertEquals(2, sender.calls.size)
            val failed = store.rowsByState(scope, OutboxState.FAILED).single()
            assertEquals(createId, failed.id)
            val blocked = store.rowsByState(scope, OutboxState.BLOCKED).single()
            assertEquals(colorId, blocked.id)
            assertTrue(store.pendingOrdered(scope).none { it.id == otherEntityId })
        }

    @Test
    fun drainOnAnEmptyOutboxStillCreatesClientStateSoThePurgeSweepSeesTheScope() =
        runTest {
            val signedIn = signedInTestStore()
            val store = signedIn.store
            val session = signedIn.session
            val sender = FakeOutboxSender()
            val worker = startedWorker(store, sender, signedIn.registry)

            worker.requestDrain()
            runCurrent()

            assertEquals(listOf(scope to false), store.scopesWithState())
            assertTrue(sender.calls.isEmpty())
        }

    @Test
    fun createInBackoffShieldsItsDueDependantAndTheDependantIsNotSent() =
        runTest {
            val signedIn = signedInTestStore()
            val store = signedIn.store
            val session = signedIn.session
            val sender = FakeOutboxSender()
            val clock = FakeClock(now = 1_000)
            val worker = startedWorker(store, sender, signedIn.registry, clock::current)
            val createId = store.enqueueHighlightCreate(session, "hlt_1", "doc_1")
            store.markAttempt(scope, createId, now = 900, nextAttemptAt = 5_000, error = "500")
            store.enqueueHighlightColor(session, "hlt_1", "doc_1")

            worker.requestDrain()
            runCurrent()

            assertTrue(sender.calls.isEmpty())
            assertEquals(5_000L, eligibleRows(store.pendingOrdered(scope), clock.current()).earliestWaitingDeadline)
        }

    @Test
    fun createDueAgainRunsBeforeItsDependantInSeqOrder() =
        runTest {
            val signedIn = signedInTestStore()
            val store = signedIn.store
            val session = signedIn.session
            val sender = FakeOutboxSender()
            val clock = FakeClock(now = 6_000)
            val worker = startedWorker(store, sender, signedIn.registry, clock::current)
            val createId = store.enqueueHighlightCreate(session, "hlt_1", "doc_1")
            store.markAttempt(scope, createId, now = 900, nextAttemptAt = 5_000, error = "500")
            val colorId = store.enqueueHighlightColor(session, "hlt_1", "doc_1")
            sender.enqueueOutcome(SendOutcome.Success)
            sender.enqueueOutcome(SendOutcome.Success)

            worker.requestDrain()
            runCurrent()

            assertEquals(listOf(createId, colorId), sender.calls.map { it.single().id })
            assertTrue(store.pendingOrdered(scope).isEmpty())
        }

    @Test
    fun terminalCreateBlocksItsPendingDependantsInOneStep() =
        runTest {
            val signedIn = signedInTestStore()
            val store = signedIn.store
            val session = signedIn.session
            val sender = FakeOutboxSender()
            val worker = startedWorker(store, sender, signedIn.registry)
            store.enqueueHighlightCreate(session, "hlt_1", "doc_1")
            val colorId = store.enqueueHighlightColor(session, "hlt_1", "doc_1")
            sender.enqueueOutcome(SendOutcome.Http(422, null, "anchor invalid"))

            worker.requestDrain()
            runCurrent()

            assertEquals(1, sender.calls.size)
            assertEquals(listOf(colorId), store.rowsByState(scope, OutboxState.BLOCKED).map { it.id })
        }
}
