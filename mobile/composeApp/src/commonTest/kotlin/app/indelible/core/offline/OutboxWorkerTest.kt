package app.indelible.core.offline

import app.indelible.db.testOfflineDatabase
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.launch
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue

class OutboxWorkerTest {
    private val scope = "scope"

    @Test
    fun secondDrainReturnsImmediatelyAndRerunsOnceTheFirstFinishes() =
        runTest {
            val store = SqlDelightOfflineStore(testOfflineDatabase())
            store.enqueueNote(scope, "doc_1")
            val sendStarted = CompletableDeferred<Unit>()
            val releaseSend = CompletableDeferred<Unit>()
            var sendCount = 0
            val sender =
                object : OutboxSender {
                    override suspend fun send(
                        scope: String,
                        batch: List<OutboxRow>,
                    ): SendOutcome {
                        sendCount++
                        if (sendStarted.complete(Unit)) releaseSend.await()
                        return SendOutcome.Success
                    }
                }
            val worker = testWorker(store, sender, scope) { 0L }

            val firstDrain = launch { worker.drain() }
            sendStarted.await()
            store.enqueueNote(scope, "doc_2")
            worker.drain()

            assertEquals(1, sendCount)

            releaseSend.complete(Unit)
            firstDrain.join()

            assertEquals(2, sendCount)
            assertTrue(store.drainable(scope, Long.MAX_VALUE).isEmpty())
        }

    @Test
    fun http401PausesAuthAndResumeAuthAllowsTheRowToDrainAgain() =
        runTest {
            val store = SqlDelightOfflineStore(testOfflineDatabase())
            val rowId = store.enqueueNote(scope, "doc_1")
            val followingId = store.enqueueNote(scope, "doc_2")
            val sender = FakeOutboxSender()
            val worker = testWorker(store, sender, scope) { 0L }
            sender.enqueueOutcome(SendOutcome.Http(401, null, "expired"))

            worker.drain()

            assertTrue(worker.authPaused.value)
            assertEquals(1, sender.calls.size)
            val pendingRows = store.rowsByState(scope, OutboxState.PENDING)
            val pending = pendingRows.single { it.id == rowId }
            assertEquals(0, pending.attempts)
            val untouched = pendingRows.single { it.id == followingId }
            assertEquals(0, untouched.attempts)

            worker.drain()
            assertEquals(1, sender.calls.size)

            worker.resumeAuth()
            assertFalse(worker.authPaused.value)
            sender.enqueueOutcome(SendOutcome.Success)
            sender.enqueueOutcome(SendOutcome.Success)
            worker.drain()

            assertTrue(store.drainable(scope, Long.MAX_VALUE).isEmpty())
        }

    @Test
    fun perEntityShieldingSkipsSiblingRowThisPassButStoreBlocksItForNext() =
        runTest {
            val store = SqlDelightOfflineStore(testOfflineDatabase())
            val sender = FakeOutboxSender()
            val worker = testWorker(store, sender, scope) { 0L }
            val createId = store.enqueueHighlightCreate(scope, "hlt_1", "doc_1")
            val colorId = store.enqueueHighlightColor(scope, "hlt_1", "doc_1")
            val otherEntityId = store.enqueueNote(scope, "doc_2")
            sender.enqueueOutcome(SendOutcome.Http(422, null, "anchor invalid"))
            sender.enqueueOutcome(SendOutcome.Success)

            worker.drain()

            assertEquals(2, sender.calls.size)
            val failed = store.rowsByState(scope, OutboxState.FAILED).single()
            assertEquals(createId, failed.id)
            val blocked = store.rowsByState(scope, OutboxState.BLOCKED).single()
            assertEquals(colorId, blocked.id)
            assertTrue(store.drainable(scope, Long.MAX_VALUE).none { it.id == otherEntityId })
        }

    @Test
    fun drainOnAnEmptyOutboxStillCreatesClientStateSoThePurgeSweepSeesTheScope() =
        runTest {
            val store = SqlDelightOfflineStore(testOfflineDatabase())
            val sender = FakeOutboxSender()
            val worker = testWorker(store, sender, scope) { 0L }

            worker.drain()

            assertEquals(listOf(scope to false), store.scopesWithState())
            assertTrue(sender.calls.isEmpty())
        }

    @Test
    fun drainWithNoScopeDoesNothing() =
        runTest {
            val store = SqlDelightOfflineStore(testOfflineDatabase())
            val sender = FakeOutboxSender()
            val worker = testWorker(store, sender, scope = null) { 0L }

            worker.drain()

            assertTrue(store.scopesWithState().isEmpty())
            assertTrue(sender.calls.isEmpty())
        }
}
