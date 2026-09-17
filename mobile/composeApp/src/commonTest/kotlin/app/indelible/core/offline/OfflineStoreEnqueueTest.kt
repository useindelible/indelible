package app.indelible.core.offline

import app.indelible.db.testOfflineDatabase
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertNull
import kotlin.test.assertTrue

class OfflineStoreEnqueueTest {
    private fun store() = SqlDelightOfflineStore(testOfflineDatabase())

    private fun noopPayload() = OutboxPayload.DocumentNote("noop", null)

    private suspend fun OfflineStore.enqueueSeq(scope: String): Long =
        enqueue(scope, OutboxKind.READING_EVENT, "doc_1", "doc_1") { noopPayload() to allocateOriginSeq() }

    @Test
    fun enqueueCommitsOutboxRowAndReturnsBuilderResult() =
        runTest {
            val store = store()
            val scope = "scope"

            val result =
                store.enqueue(scope, OutboxKind.DOCUMENT_NOTE, "doc_1", "doc_1") {
                    OutboxPayload.DocumentNote("hello", null) to "builder-result"
                }

            assertEquals("builder-result", result)
            val rows = store.pendingOrdered(scope)
            assertEquals(1, rows.size)
            val row = rows.single()
            assertEquals(OutboxKind.DOCUMENT_NOTE, row.kind)
            assertEquals("doc_1", row.entityId)
            assertEquals("doc_1", row.documentId)
            assertEquals(OutboxState.PENDING, row.state)
            assertEquals(OutboxPayload.DocumentNote("hello", null), row.payload)
        }

    @Test
    fun enqueueThrowingBuildPayloadLeavesNoOutboxRowAndDoesNotConsumeSeq() =
        runTest {
            val store = store()
            val scope = "scope"

            assertFailsWith<IllegalStateException> {
                store.enqueue(scope, OutboxKind.READING_EVENT, "doc_1", "doc_1") {
                    allocateOriginSeq()
                    error("boom")
                }
            }

            assertTrue(store.pendingOrdered(scope).isEmpty())

            val nextSeq =
                store.enqueue(scope, OutboxKind.READING_EVENT, "doc_1", "doc_1") {
                    val seq = allocateOriginSeq()
                    noopPayload() to seq
                }
            assertEquals(0L, nextSeq)
        }

    @Test
    fun enqueueUpsertCachedHighlightThenThrowRollsBackTogetherWithOutboxRow() =
        runTest {
            val store = store()
            val scope = "scope"

            assertFailsWith<IllegalStateException> {
                store.enqueue(scope, OutboxKind.HIGHLIGHT_CREATE, "hlt_1", "doc_1") {
                    upsertCachedHighlight("hlt_1", "doc_1", "{}", 1L)
                    error("boom")
                }
            }

            assertTrue(store.pendingOrdered(scope).isEmpty())
            val stillMissing =
                store.enqueue(scope, OutboxKind.HIGHLIGHT_CREATE, "hlt_1", "doc_1") {
                    val existing = getCachedHighlight("hlt_1")
                    OutboxPayload.HighlightCreate("hlt_1", "yellow", "text", null, null) to existing
                }
            assertNull(stillMissing)
        }

    @Test
    fun allocateOriginSeqIncrementsPerScope() =
        runTest {
            val store = store()
            val scope = "scope"

            val first = store.enqueueSeq(scope)
            val second = store.enqueueSeq(scope)
            val third = store.enqueueSeq(scope)

            assertEquals(listOf(0L, 1L, 2L), listOf(first, second, third))
        }

    @Test
    fun allocateOriginSeqIsIsolatedPerScope() =
        runTest {
            val store = store()

            val scopeASeq = store.enqueueSeq("scopeA")
            val scopeBSeq = store.enqueueSeq("scopeB")
            val scopeASecondSeq = store.enqueueSeq("scopeA")

            assertEquals(0L, scopeASeq)
            assertEquals(0L, scopeBSeq)
            assertEquals(1L, scopeASecondSeq)
        }

    @Test
    fun allocateOriginSeqPersistsAcrossStoreRebuild() =
        runTest {
            val database = testOfflineDatabase()
            val scope = "scope"

            val firstStore = SqlDelightOfflineStore(database)
            firstStore.enqueueSeq(scope)
            firstStore.enqueueSeq(scope)

            val rebuiltStore = SqlDelightOfflineStore(database)
            val nextSeq = rebuiltStore.enqueueSeq(scope)

            assertEquals(2L, nextSeq)
        }

    @Test
    fun capturedEnqueueTxThrowsWhenUsedAfterEnqueueReturns() =
        runTest {
            val store = store()
            val scope = "scope"
            var captured: EnqueueTx? = null

            store.enqueue(scope, OutboxKind.READING_EVENT, "doc_1", "doc_1") {
                captured = this
                noopPayload() to Unit
            }
            val seqAfterEnqueue = store.enqueueSeq(scope)

            assertFailsWith<IllegalStateException> { captured!!.allocateOriginSeq() }

            val seqAfterMisuseAttempt = store.enqueueSeq(scope)
            assertEquals(seqAfterEnqueue + 1, seqAfterMisuseAttempt)
        }

    @Test
    fun dependantEnqueuedAfterFailedCreateIsInsertedBlocked() =
        runTest {
            val store = store()
            val scope = "scope"
            val createId = store.enqueueHighlightCreate(scope, "hlt_1", "doc_1")
            store.failCreateAndBlockDependants(scope, createId, "hlt_1", "422")

            store.enqueueHighlightColor(scope, "hlt_1", "doc_1")

            assertEquals(1, store.rowsByState(scope, OutboxState.BLOCKED).size)
            assertTrue(store.pendingOrdered(scope).isEmpty())
        }

    @Test
    fun dependantEnqueuedAfterRetriedCreateIsInsertedPending() =
        runTest {
            val store = store()
            val scope = "scope"
            val createId = store.enqueueHighlightCreate(scope, "hlt_1", "doc_1")
            store.failCreateAndBlockDependants(scope, createId, "hlt_1", "422")
            store.retryRow(scope, createId)

            store.enqueueHighlightColor(scope, "hlt_1", "doc_1")

            assertEquals(2, store.pendingOrdered(scope).size)
            assertTrue(store.rowsByState(scope, OutboxState.BLOCKED).isEmpty())
        }

    @Test
    fun failedColorDoesNotBlockLaterRowsForTheSameHighlight() =
        runTest {
            val store = store()
            val scope = "scope"
            val colorId = store.enqueueHighlightColor(scope, "hlt_1", "doc_1")
            store.markFailed(scope, colorId, "400")

            store.enqueueHighlightDelete(scope, "hlt_1", "doc_1")

            assertEquals(1, store.pendingOrdered(scope).size)
            assertTrue(store.rowsByState(scope, OutboxState.BLOCKED).isEmpty())
        }

    @Test
    fun createEnqueuedAfterAFailedCreateForTheSameIdIsInsertedPending() =
        runTest {
            val store = store()
            val scope = "scope"
            val createId = store.enqueueHighlightCreate(scope, "hlt_1", "doc_1")
            store.failCreateAndBlockDependants(scope, createId, "hlt_1", "422")

            store.enqueueHighlightCreate(scope, "hlt_1", "doc_1")

            assertEquals(1, store.pendingOrdered(scope).size)
        }
}
