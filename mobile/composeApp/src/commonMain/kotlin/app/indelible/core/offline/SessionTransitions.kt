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

/**
 * The only way credentials or the server URL change. A transition is not reentrant: a block
 * never opens another transition. Under the transition lock it first checks the epoch the
 * caller captured, so a result that arrived after a newer sign-in is rejected before any side
 * effect; then it advances the epoch and withdraws the session in one swap, waits for the
 * running drain pass, lets every committed write land, runs the block, and publishes whatever
 * session the stored credentials now describe at that same epoch. Advancing the epoch when the
 * transition opens, not when it closes, is what makes a refresh that started before the
 * transition unable to write over the block's credentials.
 */
class SessionTransitions(
    private val registry: SessionRegistry,
    private val tokenStorage: TokenStorage,
    private val store: OfflineStore,
    private val worker: OutboxWorker,
) {
    private val transitionMutex = Mutex()

    fun epoch(): Long = registry.current.value.epoch

    /** [block] receives the scope that was signed in when the transition opened, if any. */
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
                // Every block writes the user id last, so clearing it first means a crash inside
                // the block leaves storage that restore() refuses instead of a mixed account.
                tokenStorage.clearUserId()
                blockStarted = true
                block(outgoingScope)
                blockStarted = false
            } finally {
                // A cancelled transition must still close: left open, it would refuse every
                // refresh and every 401 until some later transition completed.
                withContext(NonCancellable) { publishNext(next, failed = blockStarted) }
            }
            TransitionResult.Applied
        }

    /** Cold start: publish the stored session without advancing the epoch a refresh may hold. */
    suspend fun restore() {
        val session = storedSession(epoch = 0) ?: return
        store.clientIdentity(session.scope)
        registry.restore(session)
    }

    // A block that died after a partial credential write leaves storage no one can vouch for, so
    // the credentials go and no session is published; a scope whose identity cannot be created
    // publishes no session either, since nothing could be enqueued for it.
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

    // A partially cleared store (a crash between clears, or a user id saved before any token)
    // must not come back as a session, so a credential is required alongside the user id.
    private suspend fun storedSession(epoch: Long): Session? {
        val serverUrl = tokenStorage.getServerUrl()
        val userId = tokenStorage.getUserId()
        val hasCredential = tokenStorage.getToken() != null || tokenStorage.getRefreshToken() != null
        if (serverUrl == null || userId == null || !hasCredential) return null
        val origin = normalizedOrigin(serverUrl)
        return Session(epoch, origin, sessionScope(origin, userId))
    }
}
