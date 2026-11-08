package app.indelible.core.offline

import kotlinx.coroutines.flow.first
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNull
import kotlin.test.assertTrue

private const val SERVER_AT = 1_000L
private const val THEIRS_AT = 2_000L

private fun conflict(
    body: String = "theirs",
    at: Long = THEIRS_AT,
) = SyncFailure(SyncFailure.Kind.NOTE_CONFLICT, serverNoteBody = body, serverNoteUpdatedAtEpochMs = at)

class OfflineStoreNoteResolutionTest {
    /** A note edit the way the reader queues it: the copy shows the new text at once. */
    private suspend fun SignedIn.edit(body: String): String {
        store.enqueue(session, OutboxKind.DOCUMENT_NOTE, VIEW_DOC, VIEW_DOC) {
            setCachedNote(VIEW_DOC, body)
            OutboxPayload.DocumentNote(body, cachedServerNoteUpdatedAt(VIEW_DOC)) to Unit
        }
        return store.pendingOrdered(scope).last().id
    }

    private suspend fun conflicted(): Pair<SignedIn, String> {
        val s = signedInStore()
        s.installNow(serverDocument(note = "server"))
        val mine = s.edit("mine")
        s.store.markAttempt(s.scope, mine, now = 5L, nextAttemptAt = 9L, error = "note changed")
        s.store.markFailed(s.scope, mine, conflict())
        return s to mine
    }

    private suspend fun SignedIn.cachedCopy() = checkNotNull(store.cachedDocument(scope, VIEW_DOC))

    @Test
    fun keepServerNoteWritesTheServerTimestampAndRebuildsTheNoteOverLiveRows() =
        runTest {
            val (s, mine) = conflicted()
            val later = s.edit("later")
            val before = s.revision()

            assertTrue(s.store.keepServerNote(s.session, mine, THEIRS_AT))

            assertEquals(listOf(later), s.store.pendingOrdered(s.scope).map { it.id })
            assertEquals(THEIRS_AT, s.cachedCopy().noteServerUpdatedAt)
            assertEquals("later", s.cachedCopy().noteBody)
            assertEquals(before.content + 1, s.revision().content)
        }

    @Test
    fun keepServerNoteWithoutLaterEditsShowsTheServersNote() =
        runTest {
            val (s, mine) = conflicted()

            assertTrue(s.store.keepServerNote(s.session, mine, THEIRS_AT))

            val rows = s.store.observeOutbox(s.scope).first()
            assertEquals("theirs", s.cachedCopy().noteBody)
            assertTrue(rows.isEmpty())
        }

    @Test
    fun keepLocalNoteRebasesThePayloadAndResetsForRetry() =
        runTest {
            val (s, mine) = conflicted()

            assertTrue(s.store.keepLocalNote(s.session, mine, THEIRS_AT))

            val row = s.row(mine)
            assertEquals(OutboxState.PENDING, row.state)
            assertEquals(OutboxPayload.DocumentNote("mine", THEIRS_AT), row.payload)
            assertEquals(0, row.attempts)
            assertNull(row.failure)
            assertEquals("mine", s.cachedCopy().noteBody)
        }

    @Test
    fun keepLocalNoteRefusesANonConflictRow() =
        runTest {
            val s = signedInStore()
            s.installNow(serverDocument(note = "server"))
            val mine = s.edit("mine")
            s.store.markFailed(s.scope, mine, rejected("too long"))

            assertFalse(s.store.keepLocalNote(s.session, mine, THEIRS_AT))
            assertFalse(s.store.keepServerNote(s.session, mine, THEIRS_AT))

            assertEquals(OutboxState.FAILED, s.row(mine).state)
            assertEquals(OutboxPayload.DocumentNote("mine", SERVER_AT), s.row(mine).payload)
        }

    @Test
    fun aResolutionIsRefusedWhenTheRowWasSupersededMeanwhile() =
        runTest {
            val (s, mine) = conflicted()
            s.store.remove(s.scope, s.edit("newer"), ackedNoteUpdatedAt = 3_000L)
            assertTrue(s.row(mine).superseded)

            assertFalse(s.store.keepLocalNote(s.session, mine, THEIRS_AT))
            assertFalse(s.store.keepServerNote(s.session, mine, THEIRS_AT))

            assertEquals(OutboxState.FAILED, s.row(mine).state)
            assertEquals("newer", s.cachedCopy().noteBody)
        }

    @Test
    fun aResolutionIsRefusedWhenTheServerVersionMoved() =
        runTest {
            val (s, mine) = conflicted()

            assertFalse(s.store.keepLocalNote(s.session, mine, seenServerUpdatedAt = 1_500L))
            assertFalse(s.store.keepServerNote(s.session, mine, seenServerUpdatedAt = 1_500L))

            assertEquals(OutboxState.FAILED, s.row(mine).state)
            assertEquals("mine", s.cachedCopy().noteBody)
        }
}
