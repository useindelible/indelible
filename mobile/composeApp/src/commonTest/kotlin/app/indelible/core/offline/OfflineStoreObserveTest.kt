package app.indelible.core.offline

import app.indelible.db.testOfflineDatabase
import kotlinx.coroutines.channels.Channel
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
            val store = store()
            val scope = "scope"
            val emissions = Channel<List<OutboxRow>>(Channel.UNLIMITED)
            backgroundScope.launch { store.observeOutbox(scope).collect { emissions.send(it) } }

            assertTrue(emissions.receive().isEmpty())

            store.enqueue(scope, OutboxKind.DOCUMENT_NOTE, "doc_1", "doc_1") {
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
            val store = store()
            val scope = "scope"
            val emissions = Channel<List<OutboxRow>>(Channel.UNLIMITED)
            backgroundScope.launch {
                store.observeOutboxForDocument(scope, "doc_1").collect { emissions.send(it) }
            }
            assertTrue(emissions.receive().isEmpty())

            // an insert for a different document still triggers the table-wide listener, but the
            // filtered query result stays empty since it recomputes against document_id = doc_1
            store.enqueue(scope, OutboxKind.DOCUMENT_NOTE, "doc_2", "doc_2") {
                OutboxPayload.DocumentNote("other-doc", null) to Unit
            }
            assertTrue(emissions.receive().isEmpty())

            store.enqueue(scope, OutboxKind.DOCUMENT_NOTE, "doc_1", "doc_1") {
                OutboxPayload.DocumentNote("target-doc", null) to Unit
            }
            val afterTargetInsert = emissions.receive()
            assertEquals(1, afterTargetInsert.size)
            assertEquals("doc_1", afterTargetInsert.single().documentId)
        }
}
