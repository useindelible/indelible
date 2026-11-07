package app.indelible.core.offline

import app.indelible.db.testOfflineDatabase
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull
import kotlin.test.assertTrue

class OfflineStoreRetryTest {
    private fun store() = SqlDelightOfflineStore(testOfflineDatabase())

    @Test
    fun retryRowResetsFailedRowAndUnblocksSameEntityRows() =
        runTest {
            val signedIn = signedInStore()
            val store = signedIn.store
            val session = signedIn.session
            val scope = signedIn.scope
            val failedId = store.enqueueNote(session, "hlt_1", "doc_1")
            val blockedId = store.enqueueNote(session, "hlt_1", "doc_1")
            store.markAttempt(scope, failedId, now = 5L, nextAttemptAt = 999L, error = "boom")
            store.failCreateAndBlockDependants(scope, failedId, "hlt_1", "boom")
            val blockedBeforeRetry = store.rowsByState(scope, OutboxState.BLOCKED).single { it.id == blockedId }
            assertEquals(OutboxState.BLOCKED, blockedBeforeRetry.state)

            store.retryRow(scope, failedId)

            val retried = store.rowsByState(scope, OutboxState.PENDING).single { it.id == failedId }
            assertEquals(0, retried.attempts)
            assertEquals(0L, retried.nextAttemptAt)
            assertNull(retried.lastError)
            val unblocked = store.rowsByState(scope, OutboxState.PENDING).single { it.id == blockedId }
            assertEquals(OutboxState.PENDING, unblocked.state)
            assertTrue(store.rowsByState(scope, OutboxState.BLOCKED).isEmpty())
        }

    @Test
    fun retryRowIsANoOpOnAPendingRow() =
        runTest {
            val signedIn = signedInStore()
            val store = signedIn.store
            val session = signedIn.session
            val scope = signedIn.scope
            val pendingId = store.enqueueNote(session, "doc_1")

            store.retryRow(scope, pendingId)

            val row = store.rowsByState(scope, OutboxState.PENDING).single()
            assertEquals(pendingId, row.id)
            assertEquals(0, row.attempts)
        }

    @Test
    fun retryRowIsANoOpOnABlockedRow() =
        runTest {
            val signedIn = signedInStore()
            val store = signedIn.store
            val session = signedIn.session
            val scope = signedIn.scope
            val failedId = store.enqueueNote(session, "hlt_1", "doc_1")
            val blockedId = store.enqueueNote(session, "hlt_1", "doc_1")
            store.failCreateAndBlockDependants(scope, failedId, "hlt_1", "boom")
            val blockedBefore = store.rowsByState(scope, OutboxState.BLOCKED).single { it.id == blockedId }
            assertEquals(OutboxState.BLOCKED, blockedBefore.state)

            store.retryRow(scope, blockedId)

            val stillBlocked = store.rowsByState(scope, OutboxState.BLOCKED).single { it.id == blockedId }
            assertEquals(OutboxState.BLOCKED, stillBlocked.state)
        }
}
