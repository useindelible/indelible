package app.indelible.core.offline

import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue

private const val SERVER_AT = 1_000L
private const val ACKED_AT = 2_000L

class OfflineStoreNoteAckTest {
    private suspend fun signedInWithNote(): SignedIn {
        val s = signedInStore()
        s.installNow(serverDocument(note = "server"))
        return s
    }

    private suspend fun SignedIn.noteRow(body: String) = enqueue(OutboxPayload.DocumentNote(body, SERVER_AT))

    private suspend fun SignedIn.copyNoteAt(): Long? = store.cachedDocuments(scope).single().noteServerUpdatedAt

    private suspend fun SignedIn.noteOf(id: String) = row(id).payload as OutboxPayload.DocumentNote

    private suspend fun SignedIn.baseOf(id: String): Long? = noteOf(id).baseUpdatedAtEpochMs

    @Test
    fun acknowledgingANoteStoresItsServerTimestamp() =
        runTest {
            val s = signedInWithNote()
            val a = s.noteRow("a")

            s.store.remove(s.scope, a, ackedNoteUpdatedAt = ACKED_AT)

            assertEquals(ACKED_AT, s.copyNoteAt())
            assertTrue(s.store.pendingOrdered(s.scope).isEmpty())
        }

    @Test
    fun twoOfflineEditsSyncInOrderWithoutAFalseConflict() =
        runTest {
            val s = signedInWithNote()
            val a = s.noteRow("a")
            val b = s.noteRow("b")

            s.store.remove(s.scope, a, ackedNoteUpdatedAt = ACKED_AT)

            val second = s.row(b)
            assertEquals(OutboxPayload.DocumentNote("b", ACKED_AT), second.payload)
            assertEquals(OutboxState.PENDING, second.state)
            assertFalse(second.superseded)
        }

    @Test
    fun anEditRightAfterASyncCarriesTheAckedBase() =
        runTest {
            val s = signedInWithNote()
            s.store.remove(s.scope, s.noteRow("a"), ackedNoteUpdatedAt = ACKED_AT)

            s.store.enqueue(s.session, OutboxKind.DOCUMENT_NOTE, VIEW_DOC, VIEW_DOC) {
                OutboxPayload.DocumentNote("c", cachedServerNoteUpdatedAt(VIEW_DOC)) to Unit
            }

            val c = s.store.pendingOrdered(s.scope).single()
            assertEquals(ACKED_AT, s.baseOf(c.id))
        }

    @Test
    fun anAcknowledgementWithoutATimestampLeavesTheBasesAlone() =
        runTest {
            val s = signedInWithNote()
            val a = s.noteRow("a")
            val b = s.noteRow("b")
            val other = s.store.enqueueNote(s.session, "doc_2")

            s.store.remove(s.scope, a)

            assertEquals(SERVER_AT, s.copyNoteAt())
            assertEquals(SERVER_AT, s.baseOf(b))
            assertEquals(null, s.baseOf(other))
        }
}
