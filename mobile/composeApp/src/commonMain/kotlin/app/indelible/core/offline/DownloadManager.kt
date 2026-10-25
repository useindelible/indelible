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
        var forReading: Boolean = false,
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

    // Open readers by document; a held copy is never evicted, though Remove from device still removes it.
    private val held = mutableMapOf<DocumentKey, Int>()
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

    /** A copy to read now: unpinned, past the cap, never dropped silently; it merges with any other request. */
    suspend fun requestForReading(
        session: Session,
        documentId: String,
    ) {
        request(session, documentId, pin = false, forReading = true)
    }

    /** Keeps the copy out of eviction while a reader has it open; each hold needs its own [release]. */
    suspend fun hold(
        scope: String,
        documentId: String,
    ) {
        val key = DocumentKey(scope, documentId)
        lock.withLock { held[key] = (held[key] ?: 0) + 1 }
    }

    suspend fun release(
        scope: String,
        documentId: String,
    ) {
        val key = DocumentKey(scope, documentId)
        lock.withLock {
            val left = (held[key] ?: 0) - 1
            if (left > 0) held[key] = left else held.remove(key)
        }
    }

    internal suspend fun isHeld(
        scope: String,
        documentId: String,
    ): Boolean = lock.withLock { DocumentKey(scope, documentId) in held }

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
            val freed = acquiring.withLock { copies.enforceCap(scope, heldIn(scope)) }
            if (freed) lock.withLock { autoCachePaused -= scope }
        }

    /** The launch sweep; it waits for the running acquisition, and the next one waits for it. */
    suspend fun cleanupAtLaunch() = withContext(dispatcher) { acquiring.withLock { copies.sweep() } }

    private suspend fun request(
        session: Session,
        documentId: String,
        pin: Boolean,
        forReading: Boolean = false,
    ) {
        val key = DocumentKey(session.scope, documentId)
        val wanted = pin || forReading
        lock.withLock {
            if (!wanted && session.scope in autoCachePaused) return
            // An unpinned duplicate of the running acquisition would only download the same set again; one
            // being removed is about to be cancelled, so a request made now must wait for that removal.
            val beingRemoved = key in removing || key.scope in removingScopes
            val running = active?.takeUnless { beingRemoved || !it.job.isActive }?.request
            if (!wanted && running?.let { it.key == key && it.session === session } == true) return
            val merged = (queue[key] ?: parked.remove(key))?.takeIf { it.session === session }
            queue[key] =
                merged?.also {
                    it.pin = it.pin || pin
                    it.forReading = it.forReading || forReading
                } ?: Request(session, documentId, pin, forReading)
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
            registry.current.value.session !== request.session -> finish(key)
            // A row whose files are missing or cut short is repaired by a fresh generation, not trusted.
            copies.isComplete(key.scope, request.documentId) -> {
                if (request.pin) store.setPinned(key.scope, request.documentId, pinned = true)
                finish(key)
            }
            !isOnline && request.forReading -> show(request, Acquisition.Failed(DownloadFailure.NETWORK))
            !isOnline -> if (request.pin) park(request, Acquisition.WaitingForConnection) else finish(key)
            else -> fetch(request)
        }
    }

    private suspend fun fetch(request: Request) {
        val key = request.key
        acquisitions.set(key, Acquisition.Downloading(null))
        runCatching {
            // A pinned row being repaired keeps its copy whatever the cap says, as its first download did.
            val pinned = store.cachedDocument(key.scope, request.documentId)?.pinned == true
            val uncapped = request.forReading || pinned
            val outcome =
                fetcher.download(request.session, request.documentId, request.pin, ignoreCap = uncapped) {
                    acquisitions.set(key, Acquisition.Downloading(it))
                }
            if (outcome == FetchResult.Installed) copies.enforceCap(key.scope, heldIn(key.scope) + key.documentId)
            outcome
        }.onSuccess { outcome ->
            when {
                outcome == FetchResult.Installed -> {
                    lock.withLock { autoCachePaused -= key.scope }
                    finish(key)
                }
                // A reader waiting for the copy is told, not left waiting for a retry nothing schedules.
                request.forReading -> show(request, Acquisition.Failed(DownloadFailure.SERVER))
                // A pin waits for a readable render rather than claiming a copy the reader cannot open.
                outcome == FetchResult.Stale || request.pin -> park(request, Acquisition.Waiting)
                else -> finish(key)
            }
        }.onFailure { error ->
            if (error is CancellationException) throw error
            if (error is NoSpaceException) lock.withLock { autoCachePaused += key.scope }
            val shown = failureState(error, request.pin, request.forReading, isOnline)
            when {
                shown == null -> finish(key)
                request.forReading -> show(request, shown)
                isConnectionFailure(error) || keepsThePin(error) -> park(request, shown)
                else -> acquisitions.set(key, shown)
            }
        }
    }

    // A request queued behind this one for the same document takes the board over instead of seeing it go dark.
    private suspend fun finish(key: DocumentKey) {
        lock.withLock { if (key in queue) acquisitions.set(key, Acquisition.Waiting) else acquisitions.clear(key) }
    }

    // A failure shown to a reader still parks the request when it also carries a pin, so the pin is retried.
    private suspend fun show(
        request: Request,
        failure: Acquisition,
    ) {
        if (request.pin) park(request, failure) else acquisitions.set(request.key, failure)
    }

    private suspend fun heldIn(scope: String): Set<String> =
        lock.withLock { held.keys.filter { it.scope == scope }.mapTo(mutableSetOf()) { it.documentId } }

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
    forReading: Boolean,
    online: Boolean,
): Acquisition? =
    when {
        error is StaleSessionException || error is StaleWriteException || error is ScopeNotLiveException -> null
        error is NoSpaceException -> Acquisition.Failed(DownloadFailure.NO_SPACE)
        (isConnectionFailure(error) || error is ChapterListUnavailableException) && !pin && !forReading -> null
        isConnectionFailure(error) && (online || forReading) -> Acquisition.Failed(DownloadFailure.NETWORK)
        isConnectionFailure(error) -> Acquisition.WaitingForConnection
        error is ServerTooOldException -> Acquisition.Failed(DownloadFailure.SERVER_TOO_OLD)
        // A pin on a book whose list is still being built waits for it, as it waits for a readable render.
        error is ChapterListPendingException -> Acquisition.Waiting
        else -> Acquisition.Failed(DownloadFailure.SERVER)
    }

// A pinned book without a list stays parked, so the pin survives until a reopen finds the list ready.
private fun keepsThePin(error: Throwable) = error is ChapterListUnavailableException && error !is ServerTooOldException

private fun isConnectionFailure(error: Throwable) = error is CapabilitiesUnknownException || isNetworkException(error)
