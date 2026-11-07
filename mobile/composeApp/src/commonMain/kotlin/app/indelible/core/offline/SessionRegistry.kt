package app.indelible.core.offline

import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock

/**
 * The account and server a write or send is bound to. Compared by identity on purpose: a
 * re-login as the same user publishes a new object, so anything still holding the old one is
 * stale even though its scope string is unchanged.
 */
class Session(
    val epoch: Long,
    val origin: String,
    val scope: String,
)

data class SessionState(
    val epoch: Long,
    val session: Session?,
    val transitioning: Boolean = false,
)

class StaleSessionException : Exception("session changed")

class StaleWriteException : Exception("session changed before the write committed")

class NoSessionException : Exception("no session is signed in")

class ScopeNotLiveException(
    scope: String,
) : Exception("scope $scope is not live")

/**
 * The single publisher of auth identity. The epoch exists from cold start and only a
 * transition advances it; the session exists only while signed in and outside a transition.
 * While a transition is open ([SessionState.transitioning]) no refresh may write and no 401
 * may invalidate: the credentials are being replaced, so both would act on a mixture of the
 * old and the new account. Every method takes a short lock and never awaits anything else,
 * so a caller that must publish or invalidate from inside a drain pass never waits on a
 * transition.
 */
class SessionRegistry {
    private val state = MutableStateFlow(SessionState(epoch = 0, session = null))
    private val publishLock = Mutex()
    val current: StateFlow<SessionState> = state.asStateFlow()

    suspend fun publish(next: SessionState) {
        publishLock.withLock { state.value = next }
    }

    /** Cold-start publication: succeeds only while nothing has been published yet. */
    suspend fun restore(session: Session): Boolean =
        publishLock.withLock {
            val now = state.value
            if (now.epoch != 0L || now.session != null) return false
            state.value = SessionState(0, session)
            true
        }

    /** Runs [write] only if the epoch the refresh started under is still current and no transition is open. */
    suspend fun publishRefreshed(
        epoch: Long,
        write: suspend () -> Unit,
    ): Boolean =
        publishLock.withLock {
            val now = state.value
            if (now.epoch != epoch || now.transitioning) return false
            write()
            true
        }

    /** Drops the session if [epoch] is still current and no transition is open; the epoch itself stays. */
    suspend fun invalidate(epoch: Long): Boolean =
        publishLock.withLock {
            val now = state.value
            if (now.epoch != epoch || now.transitioning) return false
            state.value = now.copy(session = null)
            true
        }
}
