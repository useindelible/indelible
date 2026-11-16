package app.indelible.core.offline

import kotlinx.coroutines.flow.first
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals

private const val HLT = "hlt_1"

class OfflineStoreHighlightDiscardTest {
    /** Enqueues [payload] and writes it into the copy, as the reader's enqueue does. */
    private suspend fun SignedIn.edit(payload: OutboxPayload): String {
        val row = outboxRow(0, payload)
        store.enqueue(session, row.kind, row.entityId, VIEW_DOC) {
            applyToCachedHighlight(row.entityId, VIEW_DOC, payload, at = 0L)
            payload to Unit
        }
        return store
            .observeOutbox(scope)
            .first()
            .last()
            .id
    }

    private suspend fun SignedIn.failedEdit(payload: OutboxPayload): String =
        edit(payload).also { store.markFailed(scope, it, rejected("no")) }

    private suspend fun SignedIn.colours(): List<String> = store.cachedHighlights(scope, VIEW_DOC).map { it.color }

    private suspend fun SignedIn.discard(id: String) = store.discardRows(session, listOf(id))

    @Test
    fun discardingAColourEditRestoresTheServerColour() =
        runTest {
            val s = signedInStore()
            s.installNow(serverDocument(highlights = listOf(serverHighlight(HLT, color = "yellow"))))
            val failed = s.failedEdit(OutboxPayload.HighlightColor(HLT, "red"))
            assertEquals(listOf("red"), s.colours())

            s.discard(failed)

            assertEquals(listOf("yellow"), s.colours())
        }

    @Test
    fun discardingADeleteBringsTheHighlightBack() =
        runTest {
            val s = signedInStore()
            s.installNow(serverDocument(highlights = listOf(serverHighlight(HLT, tags = listOf("kept")))))
            val failed = s.failedEdit(OutboxPayload.HighlightDelete(HLT))
            assertEquals(emptyList(), s.colours())

            s.discard(failed)

            assertEquals(listOf(listOf("kept")), s.store.cachedHighlights(s.scope, VIEW_DOC).map { it.tags })
        }

    @Test
    fun aLaterQueuedEditStaysOnTopOfTheRestoredValue() =
        runTest {
            val s = signedInStore()
            s.installNow(serverDocument(highlights = listOf(serverHighlight(HLT, color = "yellow"))))
            val failed = s.failedEdit(OutboxPayload.HighlightNote(HLT, "mine"))
            s.edit(OutboxPayload.HighlightColor(HLT, "green"))

            s.discard(failed)

            val highlight = s.store.cachedHighlights(s.scope, VIEW_DOC).single()
            assertEquals("green", highlight.color)
            assertEquals(null, highlight.note)
        }

    @Test
    fun anAcknowledgedEditIsTheValueADiscardReturnsTo() =
        runTest {
            val s = signedInStore()
            s.installNow(serverDocument(highlights = listOf(serverHighlight(HLT, color = "yellow"))))
            s.store.remove(s.scope, s.edit(OutboxPayload.HighlightColor(HLT, "red")))
            val failed = s.failedEdit(OutboxPayload.HighlightColor(HLT, "green"))

            s.discard(failed)

            assertEquals(listOf("red"), s.colours())
        }

    @Test
    fun aRefreshedServerColourIsTheValueADiscardReturnsTo() =
        runTest {
            val s = signedInStore()
            s.installNow(serverDocument(highlights = listOf(serverHighlight(HLT, color = "yellow"))))
            val failed = s.failedEdit(OutboxPayload.HighlightColor(HLT, "red"))
            val part = ServerPart.Highlights(listOf(serverHighlight(HLT, color = "blue")))
            s.store.refreshCachedCopy(s.session, RefreshRequest(VIEW_DOC, s.revision(), part, at = REFRESH_AT))
            assertEquals(listOf("red"), s.colours())

            s.discard(failed)

            assertEquals(listOf("blue"), s.colours())
        }
}
