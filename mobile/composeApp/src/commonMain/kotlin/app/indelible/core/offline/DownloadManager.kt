package app.indelible.core.offline

import app.indelible.share.isNetworkException
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.CoroutineStart
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import kotlinx.coroutines.cancelAndJoin
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.channels.consumeEach
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.launch
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock

/**
 * Acquires offline copies one at a time. Every request is bound to the session that made it and
 * is dropped once that session is gone; duplicate requests for a document merge, and a pin wins
 * over an auto-cache. Removal and purge cancel and join the document's acquisition before any
 * row or file is deleted, so nothing an acquisition writes can outlive them.
 */
class DownloadManager(
    private val registry: SessionRegistry,
    private val fetcher: OfflineSetFetcher,
    private val store: OfflineStore,
    private val files: OfflineFiles,
    private val online: Flow<Boolean>,
    private val dispatcher: CoroutineDispatcher = Dispatchers.Default,
) {
    private class Request(
        val session: Session,
        val documentId: String,
        var pin: Boolean,
    ) {
        val key: DocumentKey get() = DocumentKey(session.scope, documentId)
    }

    private class Active(
        val request: Request,
        val job: Job,
    )

    val acquisitions = AcquisitionBoard()

    private val lock = Mutex()
    private val queue = LinkedHashMap<DocumentKey, Request>()

    // Requests parked until connectivity returns or a new request for the same document arrives.
    private val parked = LinkedHashMap<DocumentKey, Request>()
    private val autoCachePaused = mutableSetOf<String>()
    private var active: Active? = null
    private var isOnline = true
    private val wake = Channel<Unit>(Channel.CONFLATED)
    private var scope: CoroutineScope? = null

    fun start() {
        if (scope != null) return
        val owned = CoroutineScope(SupervisorJob() + dispatcher)
        scope = owned
        owned.launch { online.collect { onConnectivity(it) } }
        owned.launch { wake.consumeEach { drain(owned) } }
    }

    fun stop() {
        scope?.cancel()
        scope = null
    }

    /** Pins an existing copy, or acquires a pinned one. */
    suspend fun keepOffline(
        session: Session,
        documentId: String,
    ) {
        if (store.cachedDocument(session.scope, documentId) != null) {
            store.setPinned(session.scope, documentId, pinned = true)
        } else {
            request(session, documentId, pin = true)
        }
    }

    /** An unpinned copy of a document opened online, unless a full disk paused auto-caching. */
    suspend fun cacheOnOpen(
        session: Session,
        documentId: String,
    ) {
        request(session, documentId, pin = false)
    }

    suspend fun removeFromDevice(
        scope: String,
        documentId: String,
    ) {
        val key = DocumentKey(scope, documentId)
        val job =
            lock.withLock {
                queue.remove(key)
                parked.remove(key)
                active?.takeIf { it.request.key == key }?.job
            }
        job?.cancelAndJoin()
        acquisitions.clear(key)
        store.removeCachedDocument(scope, documentId)
        files.deleteTree(files.documentDir(scope, documentId))
    }

    /** Deletes every copy of [scope]; queued outbox rows are untouched. */
    suspend fun removeAllDownloads(scope: String) {
        cancelAndJoin(scope)
        store.cachedDocuments(scope).forEach { store.removeCachedDocument(scope, it.documentId) }
        files.deleteTree(files.scopeDir(scope))
        lock.withLock { autoCachePaused -= scope }
    }

    /** The scope purge's file step: no acquisition of [scope] can write once this returns. */
    suspend fun purge(scope: String) {
        cancelAndJoin(scope)
        files.deleteTree(files.scopeDir(scope))
        lock.withLock { autoCachePaused -= scope }
    }

    suspend fun cancelAndJoin(scope: String) {
        val job =
            lock.withLock {
                queue.keys.removeAll { it.scope == scope }
                parked.keys.removeAll { it.scope == scope }
                active?.takeIf { it.request.key.scope == scope }?.job
            }
        job?.cancelAndJoin()
        acquisitions.clearScope(scope)
    }

    private suspend fun request(
        session: Session,
        documentId: String,
        pin: Boolean,
    ) {
        val key = DocumentKey(session.scope, documentId)
        lock.withLock {
            if (!pin && session.scope in autoCachePaused) return
            val merged = (queue[key] ?: parked.remove(key))?.takeIf { it.session === session }
            queue[key] = merged?.also { it.pin = it.pin || pin } ?: Request(session, documentId, pin)
            if (active?.request?.key != key) acquisitions.set(key, Acquisition.Waiting)
        }
        wake.trySend(Unit)
    }

    private suspend fun drain(owned: CoroutineScope) {
        while (true) {
            val job =
                lock.withLock {
                    val request = queue.values.firstOrNull() ?: return
                    queue.remove(request.key)
                    val job = owned.launch(start = CoroutineStart.LAZY) { acquire(request) }
                    active = Active(request, job)
                    job
                }
            job.start()
            job.join()
            lock.withLock { if (active?.job === job) active = null }
        }
    }

    private suspend fun acquire(request: Request) {
        val key = request.key
        when {
            registry.current.value.session !== request.session -> acquisitions.clear(key)
            !isOnline -> if (request.pin) park(request, Acquisition.WaitingForConnection) else acquisitions.clear(key)
            store.cachedDocument(key.scope, request.documentId) != null -> {
                if (request.pin) store.setPinned(key.scope, request.documentId, pinned = true)
                acquisitions.clear(key)
            }
            else -> fetch(request)
        }
    }

    private suspend fun fetch(request: Request) {
        val key = request.key
        acquisitions.set(key, Acquisition.Downloading(0f))
        runCatching {
            fetcher.download(request.session, request.documentId, request.pin) { fraction ->
                acquisitions.set(key, Acquisition.Downloading(fraction))
            }
        }.onSuccess { outcome ->
            if (outcome == FetchResult.Stale) park(request, Acquisition.Waiting) else acquisitions.clear(key)
        }.onFailure { error ->
            if (error is CancellationException) throw error
            if (error is NoSpaceException) lock.withLock { autoCachePaused += key.scope }
            val shown = failureState(error, request.pin, isOnline)
            when {
                shown == null -> acquisitions.clear(key)
                isNetworkException(error) -> park(request, shown)
                else -> acquisitions.set(key, shown)
            }
        }
    }

    private suspend fun park(
        request: Request,
        acquisition: Acquisition,
    ) {
        lock.withLock { parked[request.key] = request }
        acquisitions.set(request.key, acquisition)
    }

    private suspend fun onConnectivity(nowOnline: Boolean) {
        val resumed =
            lock.withLock {
                isOnline = nowOnline
                if (!nowOnline || parked.isEmpty()) return@withLock false
                val live = registry.current.value.session
                parked.values.filter { it.session === live }.forEach { request ->
                    queue[request.key] = request
                    acquisitions.set(request.key, Acquisition.Waiting)
                }
                parked.clear()
                true
            }
        if (resumed) wake.trySend(Unit)
    }
}

/**
 * What a failed acquisition shows, or null when it simply ends: a withdrawn session or scope was
 * not a failure of the download, and an auto-cache that lost the network just waits for the next
 * open. A pin that lost the network waits for connectivity.
 */
private fun failureState(
    error: Throwable,
    pin: Boolean,
    online: Boolean,
): Acquisition? =
    when {
        error is StaleSessionException || error is StaleWriteException || error is ScopeNotLiveException -> null
        error is NoSpaceException -> Acquisition.Failed(DownloadFailure.NO_SPACE)
        isNetworkException(error) && !pin -> null
        isNetworkException(error) && online -> Acquisition.Failed(DownloadFailure.NETWORK)
        isNetworkException(error) -> Acquisition.WaitingForConnection
        else -> Acquisition.Failed(DownloadFailure.SERVER)
    }
