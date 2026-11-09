package app.indelible.core.offline

import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue

private const val SERVER_AT = 1_000L
private const val ACKED_AT = 2_000L
private const val MOVED_AT = 1_500L

private fun conflict(at: Long) =
    SyncFailure(SyncFailure.Kind.NOTE_CONFLICT, serverNoteBody = "theirs", serverNoteUpdatedAtEpochMs = at)

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
                OutboxPayload.DocumentNote("c", noteBase(VIEW_DOC)) to Unit
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

    private suspend fun SignedIn.serverNoteMovedTo(at: Long) {
        val part = ServerPart.Note(ServerNote("theirs", at))
        store.refreshCachedCopy(session, RefreshRequest(VIEW_DOC, revision(), part, at = REFRESH_AT))
    }

    private suspend fun SignedIn.editWithBase(body: String): String {
        store.enqueue(session, OutboxKind.DOCUMENT_NOTE, VIEW_DOC, VIEW_DOC) {
            OutboxPayload.DocumentNote(body, noteBase(VIEW_DOC)) to Unit
        }
        return store.pendingOrdered(scope).last().id
    }

    @Test
    fun anEditOverAQueuedNoteKeepsThatNotesBase() =
        runTest {
            val s = signedInWithNote()
            val a = s.editWithBase("a")
            s.serverNoteMovedTo(MOVED_AT)

            val b = s.editWithBase("b")

            assertEquals(SERVER_AT, s.baseOf(a))
            assertEquals(SERVER_AT, s.baseOf(b))
        }

    @Test
    fun anEditWithNothingQueuedTakesTheCopysStamp() =
        runTest {
            val s = signedInWithNote()
            s.serverNoteMovedTo(MOVED_AT)

            val c = s.editWithBase("c")

            assertEquals(MOVED_AT, s.baseOf(c))
        }

    @Test
    fun anAckResetsALaterConflictRowToBeCheckedAgain() =
        runTest {
            val s = signedInWithNote()
            val a = s.noteRow("a")
            val c = s.noteRow("c")
            s.store.markFailed(s.scope, c, conflict(MOVED_AT))

            s.store.remove(s.scope, a, ackedNoteUpdatedAt = ACKED_AT)

            val later = s.row(c)
            assertEquals(OutboxState.PENDING, later.state)
            assertEquals(null, later.failure)
            assertEquals(ACKED_AT, s.baseOf(c))
        }

    @Test
    fun anAcknowledgedNoteBecomesTheServersBody() =
        runTest {
            val s = signedInWithNote()
            s.store.remove(s.scope, s.noteRow("a"), ackedNoteUpdatedAt = ACKED_AT)
            val rejected = s.noteRow("b")
            s.store.markFailed(s.scope, rejected, rejected("too long"))

            s.store.discardRows(s.session, listOf(rejected))

            assertEquals("a", s.store.cachedDocument(s.scope, VIEW_DOC)?.noteBody)
        }
}
