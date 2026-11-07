package app.indelible.core.offline

import app.indelible.core.network.normalizedOrigin
import app.indelible.core.storage.TokenStorage
import kotlinx.coroutines.NonCancellable
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.coroutines.withContext

sealed interface TransitionResult {
    data object Applied : TransitionResult

    data object Rejected : TransitionResult
}

/** The only credential change path; the epoch advances before the block, so no older refresh can overwrite it. */
class SessionTransitions(
    private val registry: SessionRegistry,
    private val tokenStorage: TokenStorage,
    private val store: OfflineStore,
    private val worker: OutboxWorker,
) {
    private val transitionMutex = Mutex()

    fun epoch(): Long = registry.current.value.epoch

    suspend fun transition(
        expectedEpoch: Long,
        block: suspend (outgoingScope: String?) -> Unit,
    ): TransitionResult =
        transitionMutex.withLock {
            val before = registry.current.value
            if (before.epoch != expectedEpoch) return TransitionResult.Rejected
            val next = before.epoch + 1
            registry.publish(SessionState(next, session = null, transitioning = true))
            var blockStarted = false
            try {
                worker.beginTransition()
                store.quiesce()
                val outgoingScope = tokenStorage.currentOfflineScope()
                // Blocks write the user id last, so a crash inside one leaves storage restore() refuses.
                tokenStorage.clearUserId()
                blockStarted = true
                block(outgoingScope)
                blockStarted = false
            } finally {
                // A cancelled transition left open would refuse every refresh and 401 for good.
                withContext(NonCancellable) { publishNext(next, failed = blockStarted) }
            }
            TransitionResult.Applied
        }

    suspend fun restore() {
        val session = storedSession(epoch = 0) ?: return
        store.clientIdentity(session.scope)
        registry.restore(session)
    }

    // Credentials from a block that died mid-write go, and no session is published.
    private suspend fun publishNext(
        epoch: Long,
        failed: Boolean,
    ) {
        var session: Session? = null
        try {
            if (failed) {
                tokenStorage.clearAll()
            } else {
                session = storedSession(epoch)?.also { store.clientIdentity(it.scope) }
            }
        } finally {
            registry.publish(SessionState(epoch, session))
            worker.endTransition()
        }
    }

    // A user id without any credential is a half-cleared store, not a session.
    private suspend fun storedSession(epoch: Long): Session? {
        val serverUrl = tokenStorage.getServerUrl()
        val userId = tokenStorage.getUserId()
        val hasCredential = tokenStorage.getToken() != null || tokenStorage.getRefreshToken() != null
        if (serverUrl == null || userId == null || !hasCredential) return null
        val origin = normalizedOrigin(serverUrl)
        return Session(epoch, origin, sessionScope(origin, userId))
    }
}
