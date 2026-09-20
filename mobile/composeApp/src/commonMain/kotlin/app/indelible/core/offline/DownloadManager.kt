package app.indelible.core.offline

import app.indelible.share.isNetworkException
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.CoroutineStart
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.NonCancellable
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import kotlinx.coroutines.cancelAndJoin
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.channels.consumeEach
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.launch
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.coroutines.withContext

/** Acquires offline copies one at a time, each bound to the session that asked; duplicates merge. */
class DownloadManager(
    private val registry: SessionRegistry,
    private val fetcher: OfflineSetFetcher,
    private val store: OfflineStore,
    private val copies: OfflineCopies,
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

    // Held for each acquisition and eviction, so nothing deletes a generation still being written.
    private val acquiring = Mutex()
    private val queue = LinkedHashMap<DocumentKey, Request>()

    // Requests parked until connectivity returns or a new request for the same document arrives.
    private val parked = LinkedHashMap<DocumentKey, Request>()

    // Scopes a full disk paused; lifted wherever space is known to be free again.
    private val autoCachePaused = mutableSetOf<String>()

    // One entry per removal in progress, so a request waits for every overlapping removal to end.
    private val removing = mutableListOf<DocumentKey>()
    private val removingScopes = mutableListOf<String>()
    private var active: Active? = null
    private var isOnline = true
    private val wake = Channel<Unit>(Channel.CONFLATED)
    private var scope: CoroutineScope? = null

    fun start() {
        if (scope != null) return
        val owned = CoroutineScope(SupervisorJob() + dispatcher)
        scope = owned
        owned.launch {
            online.collect { nowOnline ->
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
        val key = DocumentKey(session.scope, documentId)
        val pinned =
            lock.withLock {
                val beingRemoved = key in removing || key.scope in removingScopes
                !beingRemoved && store.setPinned(key.scope, documentId, pinned = true)
            }
        if (!pinned) request(session, documentId, pin = true)
    }

    /** An unpinned copy of a document opened online, unless a full disk paused auto-caching. */
    suspend fun cacheOnOpen(
        session: Session,
        documentId: String,
    ) {
        request(session, documentId, pin = false)
    }

    /** A request for the document made while this runs waits until the copy is gone, then acquires anew. */
    suspend fun removeFromDevice(
        scope: String,
        documentId: String,
    ) = withContext(dispatcher) {
        val key = DocumentKey(scope, documentId)
        val job =
            lock.withLock {
                removing += key
                queue.remove(key)
                parked.remove(key)
                active?.takeIf { it.request.key == key }?.job
            }
        try {
            job?.cancelAndJoin()
            acquisitions.clear(key)
            copies.remove(scope, documentId)
        } finally {
            withContext(NonCancellable) {
                lock.withLock {
                    removing -= key
                    autoCachePaused -= scope
                }
            }
            wake.trySend(Unit)
        }
    }

    /** Unpins a copy, which stays until it is evicted; without a copy, withdraws the request. */
    suspend fun unpin(
        scope: String,
        documentId: String,
    ) {
        if (!store.setPinned(scope, documentId, pinned = false)) removeFromDevice(scope, documentId)
    }

    /** The purge's file step: once it returns, nothing of [scope] is written. Leaves the outbox alone. */
    suspend fun removeAllDownloads(scope: String) =
        withContext(dispatcher) {
            val job =
                lock.withLock {
                    removingScopes += scope
                    queue.keys.removeAll { it.scope == scope }
                    parked.keys.removeAll { it.scope == scope }
                    active?.takeIf { it.request.key.scope == scope }?.job
                }
            try {
                job?.cancelAndJoin()
                acquisitions.clearScope(scope)
                copies.removeAll(scope)
            } finally {
                withContext(NonCancellable) {
                    lock.withLock {
                        removingScopes -= scope
                        autoCachePaused -= scope
                    }
                }
                wake.trySend(Unit)
            }
        }

    /** Evicts [scope] down to the cap between acquisitions; room it frees lets auto-caching resume. */
    suspend fun enforceCap(scope: String) =
        withContext(dispatcher) {
            if (acquiring.withLock { copies.enforceCap(scope, keep = null) }) lock.withLock { autoCachePaused -= scope }
        }

    /** The launch sweep; it waits for the running acquisition, and the next one waits for it. */
    suspend fun cleanupAtLaunch() = withContext(dispatcher) { acquiring.withLock { copies.sweep() } }

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
            acquiring.withLock {
                val job =
                    lock.withLock {
                        val request =
                            queue.values.firstOrNull { it.key !in removing && it.key.scope !in removingScopes }
                                ?: return
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
            fetcher
                .download(request.session, request.documentId, request.pin) { fraction ->
                    acquisitions.set(key, Acquisition.Downloading(fraction))
                }.also { if (it == FetchResult.Installed) copies.enforceCap(key.scope, keep = request.documentId) }
        }.onSuccess { outcome ->
            when {
                outcome == FetchResult.Installed -> {
                    lock.withLock { autoCachePaused -= key.scope }
                    acquisitions.clear(key)
                }
                // A pin waits for a readable render rather than claiming a copy the reader cannot open.
                outcome == FetchResult.Stale || request.pin -> park(request, Acquisition.Waiting)
                else -> acquisitions.clear(key)
            }
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
}

/** What a failed acquisition shows, or null when it simply ends. */
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
