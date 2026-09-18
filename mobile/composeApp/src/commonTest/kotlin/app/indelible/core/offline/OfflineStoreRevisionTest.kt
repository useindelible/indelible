package app.indelible.core.offline

import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull

class OfflineStoreRevisionTest {
    @Test
    fun revisionMovesOnEnqueueAndOnAcknowledgementNotOnFailOrRetry() =
        runTest {
            val s = signedInStore()
            val start = s.revision()

            val note = s.enqueue(OutboxPayload.DocumentNote("mine", null))
            val afterEnqueue = s.revision()
            s.store.markFailed(s.scope, note, "rejected")
            s.store.retryRow(s.scope, note)
            val afterFailAndRetry = s.revision()
            s.store.remove(s.scope, note)
            val afterAck = s.revision()
            val opened = s.enqueueProgress(basisPoints = null)
            s.store.remove(s.scope, opened)
            val afterOpened = s.revision()
            val progress = s.enqueueProgress(basisPoints = 4_000)
            val afterProgress = s.revision()
            s.store.remove(s.scope, progress)
            val afterProgressAck = s.revision()

            assertEquals(start.content + 1, afterEnqueue.content)
            assertEquals(afterEnqueue, afterFailAndRetry)
            assertEquals(afterEnqueue.content + 1, afterAck.content)
            assertEquals(afterAck, afterOpened)
            assertEquals(afterAck.position + 1, afterProgress.position)
            assertEquals(afterProgress.position + 1, afterProgressAck.position)
            assertEquals(afterAck.content, afterProgressAck.content)
            assertEquals(1L, afterProgressAck.ackedProgressSeq)
        }

    @Test
    fun deleteQueuedBeforeFetchAndAcknowledgedDuringFetchIsNotResurrected() =
        runTest {
            val s = signedInStore()
            val delete = s.enqueue(OutboxPayload.HighlightDelete("hlt_1"))
            val fetchedAt = s.revision()
            s.store.remove(s.scope, delete)

            val result =
                s.store.installCachedDocument(
                    s.session,
                    installRequest(fetchedAt, serverDocument(highlights = listOf(serverHighlight("hlt_1")))),
                )

            assertEquals(InstallResult.Stale(content = true, position = false), result)
            assertNull(s.store.cachedDocument(s.scope, VIEW_DOC))
            assertEquals(emptyList(), s.store.cachedHighlights(s.scope, VIEW_DOC))
        }

    @Test
    fun noteQueuedBeforeFetchAndAcknowledgedDuringFetchIsNotReverted() =
        runTest {
            val s = signedInStore()
            val note = s.enqueue(OutboxPayload.DocumentNote("mine", null))
            val fetchedAt = s.revision()
            s.store.remove(s.scope, note)

            val result =
                s.store.installCachedDocument(s.session, installRequest(fetchedAt, serverDocument(note = "old")))

            assertEquals(InstallResult.Stale(content = true, position = false), result)
            assertNull(s.store.cachedDocument(s.scope, VIEW_DOC))
        }

    @Test
    fun progressQueuedBeforeFetchAndAcknowledgedDuringFetchIsNotReverted() =
        runTest {
            val s = signedInStore()
            val event = s.enqueueProgress(basisPoints = 6_000)
            val fetchedAt = s.revision()
            s.store.remove(s.scope, event)

            val result =
                s.store.installCachedDocument(
                    s.session,
                    installRequest(fetchedAt, serverDocument(progress = Progress(percent = 20, maxPercent = 20))),
                )

            assertEquals(InstallResult.Stale(content = false, position = true), result)
            assertNull(s.store.cachedDocument(s.scope, VIEW_DOC))
        }

    @Test
    fun refreshSkipsStalePartsAndAppliesTheRest() =
        runTest {
            val s = signedInStore()
            s.installNow(serverDocument(highlights = listOf(serverHighlight("hlt_1")), note = "kept"))
            val fetchedAt = s.revision()
            s.store.remove(s.scope, s.enqueue(OutboxPayload.DocumentNote("kept", null)))

            val result =
                s.store.refreshCachedCopy(
                    s.session,
                    RefreshRequest(
                        documentId = VIEW_DOC,
                        revision = fetchedAt,
                        server =
                            serverDocument(
                                highlights = listOf(serverHighlight("hlt_1"), serverHighlight("hlt_2")),
                                note = "stale",
                                progress = Progress(percent = 45, maxPercent = 50),
                                title = "Renamed",
                            ),
                    ),
                )

            assertEquals(RefreshResult.Applied(Staleness(content = true, position = false)), result)
            val copy = checkNotNull(s.store.cachedDocument(s.scope, VIEW_DOC))
            assertEquals("Renamed", copy.title)
            assertEquals(45, copy.progressPercent)
            assertEquals(50, copy.maxProgressPercent)
            assertEquals("kept", copy.noteBody)
            assertEquals(listOf("hlt_1"), s.store.cachedHighlights(s.scope, VIEW_DOC).map { it.id })
            assertEquals(
                RefreshResult.NoCopy,
                s.store.refreshCachedCopy(s.session, RefreshRequest("doc_other", fetchedAt, serverDocument())),
            )
        }
}
