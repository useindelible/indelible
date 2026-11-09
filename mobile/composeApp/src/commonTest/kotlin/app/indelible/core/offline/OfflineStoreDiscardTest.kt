package app.indelible.core.offline

import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.launch
import kotlinx.coroutines.test.runCurrent
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue

private const val HLT = "hlt_1"
private const val SERVER_AT = 1_000L

@OptIn(ExperimentalCoroutinesApi::class)
class OfflineStoreDiscardTest {
    private suspend fun SignedIn.discard(vararg ids: String): Int = store.discardRows(session, ids.toList())

    private suspend fun SignedIn.ids(): List<String> = store.observeOutbox(scope).first().map { it.id }

    /** A create that also wrote its highlight into the copy, as the reader's enqueue does. */
    private suspend fun SignedIn.createWithCopy(): String {
        val payload = OutboxPayload.HighlightCreate(HLT, "yellow", "quoted", null, null)
        store.enqueue(session, OutboxKind.HIGHLIGHT_CREATE, HLT, VIEW_DOC) {
            applyToCachedHighlight(HLT, VIEW_DOC, payload, at = 0L)
            payload to Unit
        }
        return ids().last()
    }

    @Test
    fun discardDeletesTheRowAndMovesTheContentRevision() =
        runTest {
            val s = signedInStore()
            val id = s.enqueue(OutboxPayload.HighlightColor(HLT, "red"))
            val before = s.revision()

            assertEquals(1, s.discard(id))

            assertTrue(s.ids().isEmpty())
            assertEquals(before.content + 1, s.revision().content)
            assertEquals(before.position, s.revision().position)
        }

    @Test
    fun discardingAFailedCreateRemovesItsDependantsAndItsCachedHighlight() =
        runTest {
            val s = signedInStore()
            s.installNow()
            val create = s.createWithCopy()
            s.store.failCreateAndBlockDependants(s.scope, create, HLT, rejected("bad locator"))
            val blocked = s.enqueue(OutboxPayload.HighlightColor(HLT, "red"))
            val otherNote = s.enqueue(OutboxPayload.DocumentNote("keep", null))
            assertEquals(OutboxState.BLOCKED, s.row(blocked).state)

            assertEquals(2, s.discard(create))

            assertEquals(listOf(otherNote), s.ids())
            assertTrue(s.store.cachedHighlights(s.scope, VIEW_DOC).isEmpty())
        }

    @Test
    fun discardingANoteRebuildsTheCachedNoteOverLiveRows() =
        runTest {
            val s = signedInStore()
            s.installNow(serverDocument(note = "server"))
            val failed = s.enqueue(OutboxPayload.DocumentNote("mine", SERVER_AT))
            s.store.markFailed(s.scope, failed, rejected("too long"))
            val later = s.enqueue(OutboxPayload.DocumentNote("later", SERVER_AT))

            s.discard(failed)
            assertEquals("later", s.store.cachedDocument(s.scope, VIEW_DOC)?.noteBody)

            s.discard(later)
            assertEquals("server", s.store.cachedDocument(s.scope, VIEW_DOC)?.noteBody)
        }

    @Test
    fun discardLeavesSupersededRowsAndTheirFlagsAlone() =
        runTest {
            val s = signedInStore()
            val older = s.enqueue(OutboxPayload.HighlightColor(HLT, "red"))
            s.store.remove(s.scope, s.enqueue(OutboxPayload.HighlightColor(HLT, "blue")))
            val live = s.enqueue(OutboxPayload.HighlightColor(HLT, "green"))
            val failed = s.enqueue(OutboxPayload.HighlightColor(HLT, "black"))
            s.store.markFailed(s.scope, failed, rejected("no"))

            s.discard(failed)

            assertTrue(s.row(older).superseded)
            assertFalse(s.row(live).superseded)
            assertEquals(OutboxState.PENDING, s.row(live).state)
        }

    @Test
    fun discardingAProgressRowMovesPositionRevNotAckedSeq() =
        runTest {
            val s = signedInStore()
            val id = s.enqueueProgress(basisPoints = 3_000)
            val before = s.revision()

            s.discard(id)

            val after = s.revision()
            assertEquals(before.position + 1, after.position)
            assertEquals(before.content, after.content)
            assertEquals(-1L, after.ackedProgressSeq)
        }

    @Test
    fun aResolutionEmitsTheDocumentChange() =
        runTest {
            val events = LocalChangeEvents()
            val seen = mutableListOf<String>()
            backgroundScope.launch { events.documentChanged.collect { seen += it } }
            runCurrent()
            val s = signedInTestStore(events = events)
            val id = s.enqueue(OutboxPayload.HighlightColor(HLT, "red"))

            s.discard(id)
            runCurrent()

            assertEquals(listOf(VIEW_DOC), seen)
        }

    @Test
    fun waitingByScopeCountsLiveRowsOnly() =
        runTest {
            val s = signedInStore(scope = "a")
            s.enqueueProgress(basisPoints = 1_000)
            s.enqueueProgress(basisPoints = 2_000)
            s.store.markFailed(s.scope, s.enqueue(OutboxPayload.HighlightColor(HLT, "red")), rejected("no"))
            s.enqueue(OutboxPayload.DocumentNote("a", null))
            s.store.remove(s.scope, s.enqueue(OutboxPayload.DocumentNote("b", null)))
            val b = s.switchTo("b")
            s.store.enqueueNote(b, "doc_9")
            s.store.setPurgePending("b", true)

            assertEquals(mapOf("a" to 1), s.store.observeWaitingByScope().first())
        }
}
