package app.indelible.core.offline

import app.indelible.reader.repository.ReaderOutboxWrites
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertNull
import kotlin.test.assertTrue

class OfflineStoreInstallTest {
    private fun SignedIn.writes() = ReaderOutboxWrites(store, testOutboxWorker())

    @Test
    fun firstInstallReplaysPendingColourAndNoteEdits() =
        runTest {
            val s = signedInStore()
            s.enqueue(OutboxPayload.HighlightColor("hlt_1", "blue"))
            s.enqueue(OutboxPayload.HighlightCreate("hlt_new", "green", "fresh", null, null))
            s.enqueue(OutboxPayload.DocumentNote("mine", null))

            val result = s.installNow(serverDocument(highlights = listOf(serverHighlight("hlt_1")), note = "server"))

            assertEquals(InstallResult.Installed, result)
            val highlights = s.store.cachedHighlights(s.scope, VIEW_DOC)
            assertEquals(listOf("hlt_1" to "blue", "hlt_new" to "green"), highlights.map { it.id to it.color })
            val copy = checkNotNull(s.store.cachedDocument(s.scope, VIEW_DOC))
            assertEquals("mine", copy.noteBody)
            assertEquals(1_000L, copy.noteServerUpdatedAt)
            assertEquals(1L, copy.generation)
            assertEquals(listOf("readable.html"), s.store.assetsForDocument(s.scope, VIEW_DOC).map { it.path })
        }

    @Test
    fun firstInstallTakesQueuedProgressOverServer() =
        runTest {
            val s = signedInStore()
            s.enqueueProgress(basisPoints = 6_000)

            s.installNow(serverDocument(progress = Progress(percent = 20, maxPercent = 20)))

            val copy = checkNotNull(s.store.cachedDocument(s.scope, VIEW_DOC))
            assertEquals(60, copy.progressPercent)
            assertEquals(60, copy.maxProgressPercent)
        }

    @Test
    fun installRejectedWhenSessionChanged() =
        runTest {
            val s = signedInStore("scopeA")
            val fetchedAt = s.revision()
            val stale = s.session
            s.switchTo("scopeB")

            assertFailsWith<StaleWriteException> { s.store.installCachedDocument(stale, installRequest(fetchedAt)) }
            assertNull(s.store.cachedDocument("scopeA", VIEW_DOC))
        }

    @Test
    fun installRejectedWhilePurgePending() =
        runTest {
            val s = signedInStore()
            val fetchedAt = s.revision()
            s.store.setPurgePending(s.scope, true)

            assertFailsWith<ScopeNotLiveException> {
                s.store.installCachedDocument(s.session, installRequest(fetchedAt))
            }
            assertNull(s.store.cachedDocument(s.scope, VIEW_DOC))
        }

    @Test
    fun reinstallKeepsPinnedAndLastSyncedAt() =
        runTest {
            val s = signedInStore()
            s.installNow(pin = true)
            s.store.markDocumentSynced(s.scope, VIEW_DOC, at = 77L)

            s.installNow(pin = false, generation = 2)

            val copy = checkNotNull(s.store.cachedDocument(s.scope, VIEW_DOC))
            assertTrue(copy.pinned)
            assertEquals(77L, copy.lastSyncedAt)
            assertEquals(2L, copy.generation)
        }

    @Test
    fun noteEditSurvivesAcknowledgementWithoutRefetch() =
        runTest {
            val s = signedInStore()
            s.installNow(serverDocument(note = "old"))

            s.writes().upsertDocumentNote(s.session, VIEW_DOC, "mine")
            s.store.remove(s.scope, s.store.pendingOrdered(s.scope).single().id)

            assertEquals("mine", s.store.cachedDocument(s.scope, VIEW_DOC)?.noteBody)
        }

    @Test
    fun readingEventPatchesCachedProgress() =
        runTest {
            val s = signedInStore()
            s.installNow(serverDocument(progress = Progress(percent = 20, maxPercent = 20)))
            val writes = s.writes()

            writes.readingEvent(s.session, VIEW_DOC, "progress", sessionId = null, progressBasisPoints = 6_000)
            val afterForward = checkNotNull(s.store.cachedDocument(s.scope, VIEW_DOC))
            writes.readingEvent(s.session, VIEW_DOC, "progress", sessionId = null, progressBasisPoints = 4_000)
            writes.readingEvent(s.session, VIEW_DOC, "opened", sessionId = null, progressBasisPoints = null)
            val afterBack = checkNotNull(s.store.cachedDocument(s.scope, VIEW_DOC))

            assertEquals(60 to 60, afterForward.progressPercent to afterForward.maxProgressPercent)
            assertEquals(40 to 60, afterBack.progressPercent to afterBack.maxProgressPercent)
        }

    @Test
    fun removeKeepsHighlightsWithLiveRows() =
        runTest {
            val s = signedInStore()
            s.enqueue(OutboxPayload.HighlightCreate("hlt_queued", "green", "fresh", null, null))
            s.installNow(serverDocument(highlights = listOf(serverHighlight("hlt_synced"))))

            s.store.removeCachedDocument(s.scope, VIEW_DOC)

            assertNull(s.store.cachedDocument(s.scope, VIEW_DOC))
            assertEquals(emptyList(), s.store.assetsForDocument(s.scope, VIEW_DOC))
            assertEquals(listOf("hlt_queued"), s.store.cachedHighlights(s.scope, VIEW_DOC).map { it.id })
            assertEquals(1, s.store.pendingOrdered(s.scope).size)
        }
}
