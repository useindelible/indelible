package app.indelible.core.offline

import app.indelible.db.testOfflineDatabase
import kotlinx.coroutines.test.runTest
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull
import kotlin.test.assertTrue

class OfflineStoreOutboxQueriesTest {
    private fun store() = SqlDelightOfflineStore(testOfflineDatabase())

    private suspend fun OfflineStore.enqueueNote(
        scope: String,
        entityId: String,
        documentId: String = entityId,
    ): String {
        enqueue(scope, OutboxKind.DOCUMENT_NOTE, entityId, documentId) {
            OutboxPayload.DocumentNote(entityId, null) to Unit
        }
        return drainable(scope, Long.MAX_VALUE).last().id
    }

    @Test
    fun drainableReturnsOnlyPendingRowsDueNowInSeqOrder() =
        runTest {
            val store = store()
            val scope = "scope"

            val firstId = store.enqueueNote(scope, "doc_1")
            val secondId = store.enqueueNote(scope, "doc_2")
            store.enqueueNote(scope, "doc_3")

            store.markAttempt(scope, secondId, now = 100L, nextAttemptAt = 500L, error = "retry-later")
            store.markFailed(scope, firstId, error = "fatal")

            val rows = store.drainable(scope, now = 100L)

            assertEquals(1, rows.size)
            assertEquals("doc_3", rows.single().entityId)
        }

    @Test
    fun rowsByStateFiltersByStateWithinScope() =
        runTest {
            val store = store()
            val scope = "scope"
            val failedId = store.enqueueNote(scope, "doc_1")
            store.enqueueNote(scope, "doc_2")
            store.markFailed(scope, failedId, "boom")

            val failedRows = store.rowsByState(scope, OutboxState.FAILED)
            val pendingRows = store.rowsByState(scope, OutboxState.PENDING)

            assertEquals(listOf(failedId), failedRows.map { it.id })
            assertEquals(1, pendingRows.size)
            assertEquals("doc_2", pendingRows.single().entityId)
        }

    @Test
    fun earliestRetryAtReturnsSmallestFutureNextAttempt() =
        runTest {
            val store = store()
            val scope = "scope"
            val firstId = store.enqueueNote(scope, "doc_1")
            val secondId = store.enqueueNote(scope, "doc_2")
            store.markAttempt(scope, firstId, now = 0L, nextAttemptAt = 700L, error = null)
            store.markAttempt(scope, secondId, now = 0L, nextAttemptAt = 300L, error = null)

            assertEquals(300L, store.earliestRetryAt(scope, now = 100L))
            assertNull(store.earliestRetryAt(scope, now = 1_000L))
        }

    @Test
    fun markAttemptIncrementsAttemptsAndRecordsError() =
        runTest {
            val store = store()
            val scope = "scope"
            val id = store.enqueueNote(scope, "doc_1")

            store.markAttempt(scope, id, now = 10L, nextAttemptAt = 20L, error = "network")

            val row = store.rowsByState(scope, OutboxState.PENDING).single()
            assertEquals(1, row.attempts)
            assertEquals(10L, row.lastAttemptAt)
            assertEquals(20L, row.nextAttemptAt)
            assertEquals("network", row.lastError)
        }

    @Test
    fun markFailedTransitionsRowToFailedState() =
        runTest {
            val store = store()
            val scope = "scope"
            val id = store.enqueueNote(scope, "doc_1")

            store.markFailed(scope, id, "unrecoverable")

            val row = store.rowsByState(scope, OutboxState.FAILED).single()
            assertEquals(OutboxState.FAILED, row.state)
            assertEquals("unrecoverable", row.lastError)
        }

    @Test
    fun blockDependantsMovesOnlyPendingRowsForEntityToBlocked() =
        runTest {
            val store = store()
            val scope = "scope"
            val firstId = store.enqueueNote(scope, "hlt_1", "doc_1")
            val secondId = store.enqueueNote(scope, "hlt_1", "doc_1")
            store.markFailed(scope, secondId, "already-failed")

            store.blockDependants(scope, "hlt_1")

            assertEquals(listOf(firstId), store.rowsByState(scope, OutboxState.BLOCKED).map { it.id })
            assertEquals(listOf(secondId), store.rowsByState(scope, OutboxState.FAILED).map { it.id })
        }

    @Test
    fun retryRowResetsFailedRowAndUnblocksSameEntityRows() =
        runTest {
            val store = store()
            val scope = "scope"
            val failedId = store.enqueueNote(scope, "hlt_1", "doc_1")
            val blockedId = store.enqueueNote(scope, "hlt_1", "doc_1")
            store.markAttempt(scope, failedId, now = 5L, nextAttemptAt = 999L, error = "boom")
            store.markFailed(scope, failedId, "boom")
            store.blockDependants(scope, "hlt_1")
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
            val store = store()
            val scope = "scope"
            val pendingId = store.enqueueNote(scope, "doc_1")

            store.retryRow(scope, pendingId)

            val row = store.rowsByState(scope, OutboxState.PENDING).single()
            assertEquals(pendingId, row.id)
            assertEquals(0, row.attempts)
        }

    @Test
    fun retryRowIsANoOpOnABlockedRow() =
        runTest {
            val store = store()
            val scope = "scope"
            store.enqueueNote(scope, "hlt_1", "doc_1")
            val blockedId = store.enqueueNote(scope, "hlt_1", "doc_1")
            store.blockDependants(scope, "hlt_1")
            val blockedBefore = store.rowsByState(scope, OutboxState.BLOCKED).single { it.id == blockedId }
            assertEquals(OutboxState.BLOCKED, blockedBefore.state)

            store.retryRow(scope, blockedId)

            val stillBlocked = store.rowsByState(scope, OutboxState.BLOCKED).single { it.id == blockedId }
            assertEquals(OutboxState.BLOCKED, stillBlocked.state)
        }

    @Test
    fun drainableOrdersBySeqEvenWhenIdAndNextAttemptAtOrderDiffer() =
        runTest {
            val database = testOfflineDatabase()
            val store = SqlDelightOfflineStore(database)
            val queries = database.offlineQueries
            val scope = "scope"

            // Insertion order fixes seq order: row-c, row-a, row-b. Ids and next_attempt_at are
            // both set in the opposite (alphabetical/ascending) order to prove ORDER BY seq is
            // driving the result, not an incidental match with either of those orderings.
            listOf("row-c", "row-a", "row-b").forEach { id ->
                queries.insertOutbox(
                    scope = scope,
                    id = id,
                    kind = "document_note",
                    entity_id = id,
                    document_id = "doc_1",
                    payload_json = Json.encodeToString<OutboxPayload>(OutboxPayload.DocumentNote(id, null)),
                    created_at = 0L,
                )
            }
            store.markAttempt(scope, "row-c", now = 0L, nextAttemptAt = 500L, error = null)
            store.markAttempt(scope, "row-a", now = 0L, nextAttemptAt = 100L, error = null)
            store.markAttempt(scope, "row-b", now = 0L, nextAttemptAt = 300L, error = null)

            val rows = store.drainable(scope, now = 1_000L)

            assertEquals(listOf("row-c", "row-a", "row-b"), rows.map { it.id })
        }

    @Test
    fun removeDeletesTheOutboxRow() =
        runTest {
            val store = store()
            val scope = "scope"
            val id = store.enqueueNote(scope, "doc_1")

            store.remove(scope, id)

            assertTrue(store.drainable(scope, Long.MAX_VALUE).isEmpty())
        }

    @Test
    fun readingEventPayloadRoundTripsThroughDrainableAndRowsByState() =
        runTest {
            val store = store()
            val scope = "scope"
            val payload =
                OutboxPayload.ReadingEvent(
                    eventId = "evt_1",
                    originSeq = 7L,
                    kind = "progress",
                    progressBasisPoints = 4200,
                    cause = "scroll",
                    sessionId = "ses_1",
                    attempt = 2,
                    positionJson = """{"x":1}""",
                    assetKind = "epub",
                    activeMs = 15_000,
                    recordedAtEpochMs = 1_700_000_000_000L,
                )

            store.enqueue(scope, OutboxKind.READING_EVENT, "doc_1", "doc_1") { payload to Unit }

            val fromDrainable = store.drainable(scope, Long.MAX_VALUE).single().payload
            val fromRowsByState = store.rowsByState(scope, OutboxState.PENDING).single().payload
            assertEquals(payload, fromDrainable)
            assertEquals(payload, fromRowsByState)
        }

    @Test
    fun readingEventPayloadWithNullableFieldsRoundTrips() =
        runTest {
            val store = store()
            val scope = "scope"
            val payload =
                OutboxPayload.ReadingEvent(
                    eventId = "evt_2",
                    originSeq = 0L,
                    kind = "open",
                    progressBasisPoints = null,
                    cause = null,
                    sessionId = null,
                    attempt = null,
                    positionJson = null,
                    assetKind = null,
                    activeMs = null,
                    recordedAtEpochMs = 0L,
                )

            store.enqueue(scope, OutboxKind.READING_EVENT, "doc_2", "doc_2") { payload to Unit }

            assertEquals(payload, store.drainable(scope, Long.MAX_VALUE).single().payload)
        }

    @Test
    fun outboxQueriesAreIsolatedByScope() =
        runTest {
            val store = store()
            store.enqueueNote("scopeA", "doc_1")

            assertTrue(store.drainable("scopeB", Long.MAX_VALUE).isEmpty())
            assertTrue(store.rowsByState("scopeB", OutboxState.PENDING).isEmpty())
            assertEquals(1, store.drainable("scopeA", Long.MAX_VALUE).size)
        }
}
