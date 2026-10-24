package app.indelible.core.offline

import app.indelible.db.testOfflineDatabase
import kotlinx.coroutines.test.runTest
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertTrue

class OfflineStoreOutboxQueriesTest {
    private fun store() = SqlDelightOfflineStore(testOfflineDatabase())

    @Test
    fun pendingOrderedReturnsEveryPendingRowInSeqOrderRegardlessOfNextAttemptAt() =
        runTest {
            val signedIn = signedInStore()
            val store = signedIn.store
            val session = signedIn.session
            val scope = signedIn.scope
            val firstId = store.enqueueNote(session, "doc_1")
            val secondId = store.enqueueNote(session, "doc_2")
            val thirdId = store.enqueueNote(session, "doc_3")
            store.markAttempt(scope, firstId, now = 10L, nextAttemptAt = 1_000_000L, error = "500")
            store.markFailed(scope, thirdId, error = "fatal")

            val ids = store.pendingOrdered(scope).map { it.id }

            assertEquals(listOf(firstId, secondId), ids)
        }

    @Test
    fun failCreateAndBlockDependantsMovesCreateToFailedAndPendingDependantsToBlockedTogether() =
        runTest {
            val signedIn = signedInStore()
            val store = signedIn.store
            val session = signedIn.session
            val scope = signedIn.scope
            val createId = store.enqueueHighlightCreate(session, "hlt_1", "doc_1")
            val colorId = store.enqueueHighlightColor(session, "hlt_1", "doc_1")
            val unrelated = store.enqueueNote(session, "doc_2")

            store.failCreateAndBlockDependants(scope, createId, "hlt_1", "422 anchor")

            assertEquals(listOf(createId), store.rowsByState(scope, OutboxState.FAILED).map { it.id })
            assertEquals(listOf(colorId), store.rowsByState(scope, OutboxState.BLOCKED).map { it.id })
            assertEquals(listOf(unrelated), store.pendingOrdered(scope).map { it.id })
            assertEquals("422 anchor", store.rowsByState(scope, OutboxState.FAILED).single().lastError)
        }

    @Test
    fun failCreateAndBlockDependantsLeavesAlreadyFailedDependantsAlone() =
        runTest {
            val signedIn = signedInStore()
            val store = signedIn.store
            val session = signedIn.session
            val scope = signedIn.scope
            val createId = store.enqueueHighlightCreate(session, "hlt_1", "doc_1")
            val failedColor = store.enqueueHighlightColor(session, "hlt_1", "doc_1")
            store.markFailed(scope, failedColor, "400")

            store.failCreateAndBlockDependants(scope, createId, "hlt_1", "422")

            val failedIds = store.rowsByState(scope, OutboxState.FAILED).map { it.id }.toSet()
            assertEquals(setOf(createId, failedColor), failedIds)
            assertTrue(store.rowsByState(scope, OutboxState.BLOCKED).isEmpty())
        }

    @Test
    fun failCreateAndBlockDependantsRollsBackWhenBlockingFails() =
        runTest {
            val database = testOfflineDatabase()
            val signedIn = signedInStore(database = database)
            val store = signedIn.store
            val session = signedIn.session
            val scope = signedIn.scope
            val createId = store.enqueueHighlightCreate(session, "hlt_1", "doc_1")
            store.enqueueHighlightColor(session, "hlt_1", "doc_1")
            val faulting = SqlDelightOfflineStore(database, failCreateHook = { throw InjectedStoreFailure() })

            assertFailsWith<InjectedStoreFailure> {
                faulting.failCreateAndBlockDependants(scope, createId, "hlt_1", "422 anchor")
            }

            assertTrue(store.rowsByState(scope, OutboxState.FAILED).isEmpty())
            assertTrue(store.rowsByState(scope, OutboxState.BLOCKED).isEmpty())
            assertEquals(2, store.pendingOrdered(scope).size)
        }

    @Test
    fun rowsByStateFiltersByStateWithinScope() =
        runTest {
            val signedIn = signedInStore()
            val store = signedIn.store
            val session = signedIn.session
            val scope = signedIn.scope
            val failedId = store.enqueueNote(session, "doc_1")
            store.enqueueNote(session, "doc_2")
            store.markFailed(scope, failedId, "boom")

            val failedRows = store.rowsByState(scope, OutboxState.FAILED)
            val pendingRows = store.rowsByState(scope, OutboxState.PENDING)

            assertEquals(listOf(failedId), failedRows.map { it.id })
            assertEquals(1, pendingRows.size)
            assertEquals("doc_2", pendingRows.single().entityId)
        }

    @Test
    fun markAttemptIncrementsAttemptsAndRecordsError() =
        runTest {
            val signedIn = signedInStore()
            val store = signedIn.store
            val session = signedIn.session
            val scope = signedIn.scope
            val id = store.enqueueNote(session, "doc_1")

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
            val signedIn = signedInStore()
            val store = signedIn.store
            val session = signedIn.session
            val scope = signedIn.scope
            val id = store.enqueueNote(session, "doc_1")

            store.markFailed(scope, id, "unrecoverable")

            val row = store.rowsByState(scope, OutboxState.FAILED).single()
            assertEquals(OutboxState.FAILED, row.state)
            assertEquals("unrecoverable", row.lastError)
        }

    @Test
    fun pendingOrderedOrdersBySeqEvenWhenIdAndNextAttemptAtOrderDiffer() =
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
                    state = "pending",
                )
            }
            store.markAttempt(scope, "row-c", now = 0L, nextAttemptAt = 500L, error = null)
            store.markAttempt(scope, "row-a", now = 0L, nextAttemptAt = 100L, error = null)
            store.markAttempt(scope, "row-b", now = 0L, nextAttemptAt = 300L, error = null)

            val rows = store.pendingOrdered(scope)

            assertEquals(listOf("row-c", "row-a", "row-b"), rows.map { it.id })
        }

    @Test
    fun removeDeletesTheOutboxRow() =
        runTest {
            val signedIn = signedInStore()
            val store = signedIn.store
            val session = signedIn.session
            val scope = signedIn.scope
            val id = store.enqueueNote(session, "doc_1")

            store.remove(scope, id)

            assertTrue(store.pendingOrdered(scope).isEmpty())
        }

    @Test
    fun readingEventPayloadRoundTripsThroughPendingOrderedAndRowsByState() =
        runTest {
            val signedIn = signedInStore()
            val store = signedIn.store
            val session = signedIn.session
            val scope = signedIn.scope
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

            store.enqueue(session, OutboxKind.READING_EVENT, "doc_1", "doc_1") { payload to Unit }

            val fromPending = store.pendingOrdered(scope).single().payload
            val fromRowsByState = store.rowsByState(scope, OutboxState.PENDING).single().payload
            assertEquals(payload, fromPending)
            assertEquals(payload, fromRowsByState)
        }

    @Test
    fun readingEventPayloadWithNullableFieldsRoundTrips() =
        runTest {
            val signedIn = signedInStore()
            val store = signedIn.store
            val session = signedIn.session
            val scope = signedIn.scope
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

            store.enqueue(session, OutboxKind.READING_EVENT, "doc_2", "doc_2") { payload to Unit }

            assertEquals(payload, store.pendingOrdered(scope).single().payload)
        }

    @Test
    fun outboxQueriesAreIsolatedByScope() =
        runTest {
            val signedIn = signedInStore("scopeA")
            val store = signedIn.store
            store.enqueueNote(signedIn.session, "doc_1")

            assertTrue(store.pendingOrdered("scopeB").isEmpty())
            assertTrue(store.rowsByState("scopeB", OutboxState.PENDING).isEmpty())
            assertEquals(1, store.pendingOrdered("scopeA").size)
        }
}
