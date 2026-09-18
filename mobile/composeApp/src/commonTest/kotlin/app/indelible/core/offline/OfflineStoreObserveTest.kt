package app.indelible.core.offline

import app.indelible.db.testOfflineDatabase
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.launch
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class OfflineStoreObserveTest {
    private fun store() = SqlDelightOfflineStore(testOfflineDatabase())

    @Test
    fun observeOutboxEmitsOnInsertAndOnRemove() =
        runTest {
            val signedIn = signedInStore()
            val store = signedIn.store
            val session = signedIn.session
            val scope = signedIn.scope
            val emissions = Channel<List<OutboxRow>>(Channel.UNLIMITED)
            backgroundScope.launch { store.observeOutbox(scope).collect { emissions.send(it) } }

            assertTrue(emissions.receive().isEmpty())

            store.enqueue(session, OutboxKind.DOCUMENT_NOTE, "doc_1", "doc_1") {
                OutboxPayload.DocumentNote("note", null) to Unit
            }
            val afterInsert = emissions.receive()
            assertEquals(1, afterInsert.size)

            store.remove(scope, afterInsert.single().id)
            assertTrue(emissions.receive().isEmpty())
        }

    @Test
    fun observeOutboxForDocumentOnlyEmitsRowsForThatDocument() =
        runTest {
            val signedIn = signedInStore()
            val store = signedIn.store
            val session = signedIn.session
            val scope = signedIn.scope
            val emissions = Channel<List<OutboxRow>>(Channel.UNLIMITED)
            backgroundScope.launch {
                store.observeOutboxForDocument(scope, "doc_1").collect { emissions.send(it) }
            }
            assertTrue(emissions.receive().isEmpty())

            // an insert for a different document still triggers the table-wide listener, but the
            // filtered query result stays empty since it recomputes against document_id = doc_1
            store.enqueue(session, OutboxKind.DOCUMENT_NOTE, "doc_2", "doc_2") {
                OutboxPayload.DocumentNote("other-doc", null) to Unit
            }
            assertTrue(emissions.receive().isEmpty())

            store.enqueue(session, OutboxKind.DOCUMENT_NOTE, "doc_1", "doc_1") {
                OutboxPayload.DocumentNote("target-doc", null) to Unit
            }
            val afterTargetInsert = emissions.receive()
            assertEquals(1, afterTargetInsert.size)
            assertEquals("doc_1", afterTargetInsert.single().documentId)
        }

    @Test
    fun catalogFlowEmitsOnInstallPinRemove() =
        runTest {
            val s = signedInStore()
            val emissions = Channel<List<CatalogEntry>>(Channel.UNLIMITED)
            backgroundScope.launch { s.store.observeCatalog(s.scope).collect { emissions.send(it) } }
            assertTrue(emissions.receive().isEmpty())

            s.installNow()
            val installed = emissions.receive().single()
            s.store.setPinned(s.scope, VIEW_DOC, pinned = true)
            val pinned = emissions.receive().single()
            s.store.removeCachedDocument(s.scope, VIEW_DOC)

            assertEquals(CatalogEntry(VIEW_DOC, "article", "Title", false, 5_000L, null, 10L, 1L), installed)
            assertTrue(pinned.pinned)
            assertTrue(emissions.receive().isEmpty())
        }

    @Test
    fun syncCountsGroupByDocument() =
        runTest {
            val s = signedInStore()
            val retrying = s.enqueue(OutboxPayload.DocumentNote("a", null))
            s.store.markAttempt(s.scope, retrying, now = 1L, nextAttemptAt = 2L, error = "503")
            s.enqueue(OutboxPayload.HighlightColor("hlt_1", "blue"))
            val failed = s.enqueue(OutboxPayload.HighlightTags("hlt_1", listOf("x")))
            s.store.markFailed(s.scope, failed, "rejected")
            s.store.enqueue(s.session, OutboxKind.DOCUMENT_NOTE, "doc_2", "doc_2") {
                OutboxPayload.DocumentNote("b", null) to Unit
            }
            val superseded = s.enqueue(OutboxPayload.HighlightNote("hlt_1", "old"))
            s.store.remove(s.scope, s.enqueue(OutboxPayload.HighlightNote("hlt_1", "new")))

            val counts = s.store.observeDocumentSyncCounts(s.scope).first()

            assertEquals(DocumentSyncCounts(pending = 2, retrying = 1, failed = 1, blocked = 0), counts[VIEW_DOC])
            assertEquals(DocumentSyncCounts(pending = 1, retrying = 0, failed = 0, blocked = 0), counts["doc_2"])
            assertTrue(s.row(superseded).superseded)
        }
}
