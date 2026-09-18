package app.indelible.core.offline

import kotlinx.coroutines.test.runCurrent
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue

private val REJECTED = SendOutcome.Http(status = 422, retryAfterSeconds = null, message = "rejected")

class OfflineStoreSupersessionTest {
    @Test
    fun olderFailedNoteDoesNotOverrideNewerAcknowledgedNote() =
        runTest {
            val s = signedInStore()
            val older = s.enqueue(OutboxPayload.DocumentNote("older", null))
            s.store.markFailed(s.scope, older, "rejected")
            s.store.remove(s.scope, s.enqueue(OutboxPayload.DocumentNote("newer", null)))

            s.installNow(serverDocument(note = "newer"))

            assertEquals("newer", s.store.localChanges(s.scope, VIEW_DOC).note("newer"))
            assertEquals("newer", s.store.cachedDocument(s.scope, VIEW_DOC)?.noteBody)
            assertTrue(s.row(older).superseded)
        }

    @Test
    fun olderFailedHighlightColourDoesNotOverrideNewerAcknowledgedColour() =
        runTest {
            val s = signedInStore()
            val older = s.enqueue(OutboxPayload.HighlightColor("hlt_1", "red"))
            s.store.markFailed(s.scope, older, "rejected")
            s.store.remove(s.scope, s.enqueue(OutboxPayload.HighlightColor("hlt_1", "blue")))

            s.installNow(serverDocument(highlights = listOf(serverHighlight("hlt_1", color = "blue"))))

            assertEquals(listOf("blue"), s.store.cachedHighlights(s.scope, VIEW_DOC).map { it.color })
        }

    @Test
    fun olderFailedProgressDoesNotOverrideNewerAcknowledgedEvent() =
        runTest {
            val s = signedInStore()
            val older = s.enqueueProgress(basisPoints = 3_000)
            s.store.markFailed(s.scope, older, "rejected")
            s.store.remove(s.scope, s.enqueueProgress(basisPoints = 6_000))

            s.installNow(serverDocument(progress = Progress(percent = 60, maxPercent = 60)))

            assertEquals(60, s.store.cachedDocument(s.scope, VIEW_DOC)?.progressPercent)
            assertFalse(s.row(older).superseded)
        }

    @Test
    fun retriedOlderReadingEventStillSends() =
        runTest {
            val s = signedInStore()
            val older = s.enqueueProgress(basisPoints = 3_000)
            s.store.markFailed(s.scope, older, "rejected")
            s.store.remove(s.scope, s.enqueueProgress(basisPoints = 6_000))

            assertTrue(s.store.retryRow(s.scope, older))
            assertEquals(listOf(older), s.store.pendingOrdered(s.scope).map { it.id })
        }

    @Test
    fun acknowledgedDeleteSupersedesOlderFailedEdits() =
        runTest {
            val s = signedInStore()
            val colour = s.enqueue(OutboxPayload.HighlightColor("hlt_1", "red"))
            val tags = s.enqueue(OutboxPayload.HighlightTags("hlt_1", listOf("a")))
            s.store.markFailed(s.scope, colour, "rejected")
            s.store.markFailed(s.scope, tags, "rejected")
            val unrelated = s.enqueue(OutboxPayload.HighlightColor("hlt_2", "red"))

            s.store.remove(s.scope, s.enqueue(OutboxPayload.HighlightDelete("hlt_1")))

            assertTrue(s.row(colour).superseded)
            assertTrue(s.row(tags).superseded)
            assertFalse(s.row(unrelated).superseded)
            assertFalse(s.store.retryRow(s.scope, colour))
            assertEquals(listOf("hlt_2"), s.store.localChanges(s.scope, VIEW_DOC).rows.map { it.entityId })
        }

    @Test
    fun supersedingIsPerFieldNotPerDocument() =
        runTest {
            val s = signedInStore()
            val tags = s.enqueue(OutboxPayload.HighlightTags("hlt_1", listOf("mine")))
            s.store.markFailed(s.scope, tags, "rejected")
            s.store.remove(s.scope, s.enqueue(OutboxPayload.HighlightColor("hlt_1", "blue")))

            s.installNow(serverDocument(highlights = listOf(serverHighlight("hlt_1", color = "blue"))))

            val cached = s.store.cachedHighlights(s.scope, VIEW_DOC).single()
            assertEquals("blue", cached.color)
            assertEquals(listOf("mine"), cached.tags)
            assertFalse(s.row(tags).superseded)
        }

    @Test
    fun retryAfterSupersessionIsRejected() =
        runTest {
            val s = signedInStore()
            val older = s.enqueue(OutboxPayload.DocumentNote("older", null))
            s.store.markFailed(s.scope, older, "rejected")
            s.store.remove(s.scope, s.enqueue(OutboxPayload.DocumentNote("newer", null)))

            assertFalse(s.store.retryRow(s.scope, older))
            assertEquals(OutboxState.FAILED, s.row(older).state)
            assertEquals(emptyList(), s.store.pendingOrdered(s.scope))
        }

    @Test
    fun retryBeforeNewerAcknowledgementIsSupersededAndNeverSent() =
        runTest {
            val s = signedInTestStore()
            val sender = GatedSender()
            val worker = startedWorker(s.store, sender, s.registry)
            sender.enqueueOutcome(REJECTED)
            val older = s.enqueue(OutboxPayload.DocumentNote("older", null))
            worker.requestDrain()
            runCurrent()
            val newer = s.enqueue(OutboxPayload.DocumentNote("newer", null))
            val inFlight = sender.holdNextSend()
            worker.requestDrain()
            runCurrent()
            inFlight.await()

            assertTrue(s.store.retryRow(s.scope, older))
            sender.release()
            runCurrent()
            worker.requestDrain()
            runCurrent()

            assertEquals(listOf(listOf(older), listOf(newer)), sender.calls.map { batch -> batch.map { it.id } })
            assertTrue(s.row(older).superseded)
            assertEquals(OutboxState.PENDING, s.row(older).state)
        }

    @Test
    fun drainExcludesSupersededRows() =
        runTest {
            val s = signedInTestStore()
            val sender = GatedSender()
            val worker = startedWorker(s.store, sender, s.registry)
            val older = s.enqueue(OutboxPayload.HighlightTags("hlt_1", listOf("old")))
            val newer = s.enqueue(OutboxPayload.HighlightTags("hlt_1", listOf("new")))
            s.store.remove(s.scope, newer)
            val other = s.enqueue(OutboxPayload.HighlightTags("hlt_2", listOf("x")))

            worker.requestDrain()
            runCurrent()

            assertEquals(listOf(other), sender.calls.flatten().map { it.id })
            assertTrue(s.row(older).superseded)
        }
}
