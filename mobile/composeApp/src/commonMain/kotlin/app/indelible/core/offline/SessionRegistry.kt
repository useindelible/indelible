package app.indelible.core.offline

import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock

/** Compared by identity: a re-login as the same user publishes a new object and makes the old one stale. */
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

/** The single publisher of auth identity; while a transition is open no refresh writes and no 401 invalidates. */
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
