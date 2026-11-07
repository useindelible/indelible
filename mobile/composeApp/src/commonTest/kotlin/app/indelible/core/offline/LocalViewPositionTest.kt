package app.indelible.core.offline

import kotlinx.coroutines.flow.first
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals

private val SERVER_AT = ChapterPosition("ch-server", 40)
private val QUEUED_AT = ChapterPosition("ch-mine", 120)

class LocalViewPositionTest {
    @Test
    fun queuedPositionWinsOverServer() =
        runTest {
            val s = signedInStore()
            s.installNow(serverDocument(progress = Progress(10, 10, SERVER_AT)))
            assertEquals(SERVER_AT, s.cachedCopy().position)

            s.enqueuePosition(QUEUED_AT)

            assertEquals(QUEUED_AT, s.cachedCopy().position)
            val view = s.store.localChanges(s.scope, VIEW_DOC).progress(Progress(10, 10, SERVER_AT))
            assertEquals(QUEUED_AT, view.position)
            val reader = ServerPart.Reader("Title", "{}", Progress(10, 10, SERVER_AT))
            s.store.refreshCachedCopy(s.session, RefreshRequest(VIEW_DOC, s.revision(), reader, REFRESH_AT))
            assertEquals(QUEUED_AT, s.cachedCopy().position)
        }

    @Test
    fun laterProgressWithoutPositionKeepsTheQueuedPosition() =
        runTest {
            val s = signedInStore()
            s.installNow(serverDocument(progress = Progress(10, 10, SERVER_AT)))
            s.enqueuePosition(QUEUED_AT)
            s.enqueueProgressOnly(7_000)

            val view = s.store.localChanges(s.scope, VIEW_DOC).progress(Progress(10, 10, SERVER_AT))

            assertEquals(Progress(70, 70, QUEUED_AT), view)
        }

    @Test
    fun progressWithoutPositionKeepsTheServerPosition() =
        runTest {
            val s = signedInStore()
            s.installNow(serverDocument(progress = Progress(10, 10, SERVER_AT)))
            s.enqueueProgressOnly(7_000)

            val view = s.store.localChanges(s.scope, VIEW_DOC).progress(Progress(10, 10, SERVER_AT))

            assertEquals(Progress(70, 70, SERVER_AT), view)
        }

    @Test
    fun acknowledgedPositionSurvivesWithoutRefetch() =
        runTest {
            val s = signedInStore()
            s.installNow(serverDocument(progress = Progress(10, 10, SERVER_AT)))
            val event = s.enqueuePosition(QUEUED_AT)

            s.store.remove(s.scope, event)

            assertEquals(QUEUED_AT, s.cachedCopy().position)
        }

    @Test
    fun stalePositionRefreshIsSkipped() =
        runTest {
            val s = signedInStore()
            s.installNow(serverDocument(progress = Progress(10, 10, SERVER_AT)))
            val fetchedAt = s.revision()
            s.store.remove(s.scope, s.enqueuePosition(QUEUED_AT))
            val reader = ServerPart.Reader("Title", "{}", Progress(10, 10, SERVER_AT))

            val stale = s.store.refreshCachedCopy(s.session, RefreshRequest(VIEW_DOC, fetchedAt, reader, REFRESH_AT))

            assertEquals(RefreshResult.Stale, stale)
            assertEquals(QUEUED_AT, s.cachedCopy().position)
            val fresh = RefreshRequest(VIEW_DOC, s.revision(), reader, REFRESH_AT)
            assertEquals(RefreshResult.Applied, s.store.refreshCachedCopy(s.session, fresh))
            assertEquals(SERVER_AT, s.cachedCopy().position)
        }

    private suspend fun SignedIn.cachedCopy() = checkNotNull(store.cachedDocument(scope, VIEW_DOC))

    private suspend fun SignedIn.enqueueProgressOnly(basisPoints: Int) {
        store.enqueue(session, OutboxKind.READING_EVENT, VIEW_DOC, VIEW_DOC) {
            progressEvent(allocateOriginSeq(), basisPoints) to Unit
        }
    }

    private suspend fun SignedIn.enqueuePosition(position: ChapterPosition): String {
        store.enqueue(session, OutboxKind.READING_EVENT, VIEW_DOC, VIEW_DOC) {
            patchCachedProgress(VIEW_DOC, 50)
            patchCachedPosition(VIEW_DOC, position)
            progressEvent(allocateOriginSeq(), 5_000, readingPositionJson(position, fraction = 0.5)) to Unit
        }
        return store
            .observeOutbox(scope)
            .first()
            .last()
            .id
    }
}
