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

    private suspend fun OfflineStore.enqueueSeq(session: Session): Long =
        enqueue(session, OutboxKind.READING_EVENT, "doc_1", "doc_1") { noopPayload() to allocateOriginSeq() }

    @Test
    fun enqueueCommitsOutboxRowAndReturnsBuilderResult() =
        runTest {
            val signedIn = signedInStore()
            val store = signedIn.store
            val session = signedIn.session
            val scope = signedIn.scope

            val result =
                store.enqueue(session, OutboxKind.DOCUMENT_NOTE, "doc_1", "doc_1") {
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
            val signedIn = signedInStore()
            val store = signedIn.store
            val session = signedIn.session
            val scope = signedIn.scope

            assertFailsWith<IllegalStateException> {
                store.enqueue(session, OutboxKind.READING_EVENT, "doc_1", "doc_1") {
                    allocateOriginSeq()
                    error("boom")
                }
            }

            assertTrue(store.pendingOrdered(scope).isEmpty())

            val nextSeq =
                store.enqueue(session, OutboxKind.READING_EVENT, "doc_1", "doc_1") {
                    val seq = allocateOriginSeq()
                    noopPayload() to seq
                }
            assertEquals(0L, nextSeq)
        }

    @Test
    fun enqueueUpsertCachedHighlightThenThrowRollsBackTogetherWithOutboxRow() =
        runTest {
            val signedIn = signedInStore()
            val store = signedIn.store
            val session = signedIn.session
            val scope = signedIn.scope

            assertFailsWith<IllegalStateException> {
                store.enqueue(session, OutboxKind.HIGHLIGHT_CREATE, "hlt_1", "doc_1") {
                    upsertCachedHighlight("hlt_1", "doc_1", "{}", 1L)
                    error("boom")
                }
            }

            assertTrue(store.pendingOrdered(scope).isEmpty())
            val stillMissing =
                store.enqueue(session, OutboxKind.HIGHLIGHT_CREATE, "hlt_1", "doc_1") {
                    val existing = getCachedHighlight("hlt_1")
                    OutboxPayload.HighlightCreate("hlt_1", "yellow", "text", null, null) to existing
                }
            assertNull(stillMissing)
        }

    @Test
    fun allocateOriginSeqIncrementsPerScope() =
        runTest {
            val signedIn = signedInStore()
            val store = signedIn.store
            val session = signedIn.session
            val scope = signedIn.scope

            val first = store.enqueueSeq(session)
            val second = store.enqueueSeq(session)
            val third = store.enqueueSeq(session)

            assertEquals(listOf(0L, 1L, 2L), listOf(first, second, third))
        }

    @Test
    fun allocateOriginSeqIsIsolatedPerScope() =
        runTest {
            val signedIn = signedInStore("scopeA")
            val store = signedIn.store

            val scopeASeq = store.enqueueSeq(signedIn.session)
            val scopeBSeq = store.enqueueSeq(signedIn.switchTo("scopeB"))
            val scopeASecondSeq = store.enqueueSeq(signedIn.switchTo("scopeA"))

            assertEquals(0L, scopeASeq)
            assertEquals(0L, scopeBSeq)
            assertEquals(1L, scopeASecondSeq)
        }

    @Test
    fun allocateOriginSeqPersistsAcrossStoreRebuild() =
        runTest {
            val database = testOfflineDatabase()
            val signedIn = signedInStore(database = database)
            val session = signedIn.session

            val firstStore = signedIn.store
            firstStore.enqueueSeq(session)
            firstStore.enqueueSeq(session)

            val rebuiltStore = SqlDelightOfflineStore(database, registry = signedIn.registry)
            val nextSeq = rebuiltStore.enqueueSeq(session)

            assertEquals(2L, nextSeq)
        }

    @Test
    fun capturedEnqueueTxThrowsWhenUsedAfterEnqueueReturns() =
        runTest {
            val signedIn = signedInStore()
            val store = signedIn.store
            val session = signedIn.session
            val scope = signedIn.scope
            var captured: EnqueueTx? = null

            store.enqueue(session, OutboxKind.READING_EVENT, "doc_1", "doc_1") {
                captured = this
                noopPayload() to Unit
            }
            val seqAfterEnqueue = store.enqueueSeq(session)

            assertFailsWith<IllegalStateException> { captured!!.allocateOriginSeq() }

            val seqAfterMisuseAttempt = store.enqueueSeq(session)
            assertEquals(seqAfterEnqueue + 1, seqAfterMisuseAttempt)
        }

    @Test
    fun dependantEnqueuedAfterFailedCreateIsInsertedBlocked() =
        runTest {
            val signedIn = signedInStore()
            val store = signedIn.store
            val session = signedIn.session
            val scope = signedIn.scope
            val createId = store.enqueueHighlightCreate(session, "hlt_1", "doc_1")
            store.failCreateAndBlockDependants(scope, createId, "hlt_1", "422")

            store.enqueueHighlightColor(session, "hlt_1", "doc_1")

            assertEquals(1, store.rowsByState(scope, OutboxState.BLOCKED).size)
            assertTrue(store.pendingOrdered(scope).isEmpty())
        }

    @Test
    fun dependantEnqueuedAfterRetriedCreateIsInsertedPending() =
        runTest {
            val signedIn = signedInStore()
            val store = signedIn.store
            val session = signedIn.session
            val scope = signedIn.scope
            val createId = store.enqueueHighlightCreate(session, "hlt_1", "doc_1")
            store.failCreateAndBlockDependants(scope, createId, "hlt_1", "422")
            store.retryRow(scope, createId)

            store.enqueueHighlightColor(session, "hlt_1", "doc_1")

            assertEquals(2, store.pendingOrdered(scope).size)
            assertTrue(store.rowsByState(scope, OutboxState.BLOCKED).isEmpty())
        }

    @Test
    fun failedColorDoesNotBlockLaterRowsForTheSameHighlight() =
        runTest {
            val signedIn = signedInStore()
            val store = signedIn.store
            val session = signedIn.session
            val scope = signedIn.scope
            val colorId = store.enqueueHighlightColor(session, "hlt_1", "doc_1")
            store.markFailed(scope, colorId, "400")

            store.enqueueHighlightDelete(session, "hlt_1", "doc_1")

            assertEquals(1, store.pendingOrdered(scope).size)
            assertTrue(store.rowsByState(scope, OutboxState.BLOCKED).isEmpty())
        }

    @Test
    fun createEnqueuedAfterAFailedCreateForTheSameIdIsInsertedPending() =
        runTest {
            val signedIn = signedInStore()
            val store = signedIn.store
            val session = signedIn.session
            val scope = signedIn.scope
            val createId = store.enqueueHighlightCreate(session, "hlt_1", "doc_1")
            store.failCreateAndBlockDependants(scope, createId, "hlt_1", "422")

            store.enqueueHighlightCreate(session, "hlt_1", "doc_1")

            assertEquals(1, store.pendingOrdered(scope).size)
        }
}
