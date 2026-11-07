package app.indelible.reader.repository

import app.indelible.core.network.LibraryApiService
import app.indelible.core.network.ReaderApiService
import app.indelible.core.offline.DL_DOC
import app.indelible.core.offline.DL_SCOPE
import app.indelible.core.offline.DownloadHarness
import app.indelible.core.offline.FakeServer
import app.indelible.core.offline.Route
import app.indelible.core.offline.SessionState
import app.indelible.core.offline.downloadHarness
import app.indelible.core.offline.startedWorker
import app.indelible.reader.model.HighlightData
import app.indelible.reader.model.ReaderDocument
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.test.TestScope
import okio.IOException

internal const val READER_PATH = "/api/v1/documents/$DL_DOC"
internal const val HIGHLIGHTS_PATH = "/api/v1/documents/$DL_DOC/highlights"
internal const val NOTE_PATH = "/api/v1/documents/$DL_DOC/note"
internal const val HTML_PATH = "/api/v1/assets/documents/$DL_DOC/readable_html"
internal const val TOC_PATH = "/api/v1/documents/$DL_DOC/toc"
internal const val READ_BUDGET_MS = 3_000L

/** A reader repository over a fake server whose document can be kept as a real offline copy. */
internal class ReaderOfflineHarness(
    val download: DownloadHarness,
    val repository: ApiReaderRepository,
) {
    val server: FakeServer get() = download.server
    val store get() = download.store

    suspend fun document(): ReaderDocument = repository.getItem(DL_DOC).getOrThrow()

    suspend fun note(): String? = repository.getItemNote(DL_DOC).getOrThrow()

    suspend fun highlights(): List<HighlightData> = repository.listHighlights(DL_DOC).getOrThrow()

    suspend fun keepCopy() {
        download.fetcher.download(download.session, DL_DOC, pin = true)
    }

    /** Every request fails the way it does with no network. */
    fun goOffline() {
        val unreachable: Route = { throw IOException("unreachable") }
        for (path in server.routes.keys.toList()) server.routes[path] = unreachable
    }

    /** Serves [path] as before, but first runs [during], the way a change landing mid-fetch would. */
    fun duringFetchOf(
        path: String,
        during: suspend () -> Unit,
    ) {
        val served = server.routes.getValue(path)
        server.route(path) { request ->
            during()
            served(request)
        }
    }

    suspend fun signOut() {
        val registry = download.signedIn.registry
        registry.publish(SessionState(registry.current.value.epoch, session = null))
    }

    /** A note edit made and acknowledged, as a sync finishing mid-fetch would: the content revision moves. */
    suspend fun acknowledgeNoteEdit() {
        repository.upsertItemNote(DL_DOC, "acknowledged")
        acknowledgeLastRow()
    }

    /** A progress event recorded and acknowledged: the position revision moves. */
    suspend fun acknowledgeProgress() {
        repository.recordProgress(DL_DOC, percent = 90f, sessionId = "s")
        acknowledgeLastRow()
    }

    /** Every queued change reaches the server, as a sync finishing mid-fetch would. */
    suspend fun acknowledgeQueued() {
        store.observeOutbox(DL_SCOPE).first().forEach { store.remove(DL_SCOPE, it.id) }
    }

    private suspend fun acknowledgeLastRow() {
        val row = store.observeOutbox(DL_SCOPE).first().last()
        store.remove(DL_SCOPE, row.id)
    }
}

internal suspend fun TestScope.readerOfflineHarness(configure: FakeServer.() -> Unit = {}): ReaderOfflineHarness {
    val download = downloadHarness(configure = configure)
    val registry = download.signedIn.registry
    val session = { registry.current.value.session }
    val offlineCopy =
        ReaderOfflineCopy(
            store = download.store,
            files = download.files,
            sessionProvider = session,
            clock = { testScheduler.currentTime },
            budgetMs = READ_BUDGET_MS,
            requestCache = { live, documentId -> download.manager.cacheOnOpen(live, documentId) },
        )
    val repository =
        ApiReaderRepository(
            readerApiService = ReaderApiService(download.transport),
            libraryApiService = LibraryApiService(download.transport),
            offlineStore = download.store,
            worker = startedWorker(download.store, StuckOutboxSender(), registry),
            sessionProvider = session,
            offlineCopy = offlineCopy,
        )
    return ReaderOfflineHarness(download, repository)
}
