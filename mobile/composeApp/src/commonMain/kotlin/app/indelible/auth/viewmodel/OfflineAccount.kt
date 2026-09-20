package app.indelible.auth.viewmodel

import app.indelible.core.model.AuthUser
import app.indelible.core.network.normalizedOrigin
import app.indelible.core.offline.OfflineStore
import app.indelible.core.offline.OutboxWorker
import app.indelible.core.offline.ScopePurger
import app.indelible.core.offline.currentOfflineScope
import app.indelible.core.offline.sessionScope
import app.indelible.core.storage.TokenStorage
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.CoroutineStart
import kotlinx.coroutines.async
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.dropWhile
import kotlinx.coroutines.flow.first
import kotlinx.serialization.json.Json

class OfflineAccount(
    private val store: OfflineStore,
    private val tokenStorage: TokenStorage,
    private val worker: OutboxWorker,
    private val purger: ScopePurger,
    private val online: Flow<Boolean>,
) {
    /** The profile the stored account last published, or null when there is none to trust. */
    suspend fun cachedUser(): AuthUser? {
        val kept = tokenStorage.currentOfflineScope()?.let { store.profile(it) } ?: return null
        return runCatching { json.decodeFromString(AuthUser.serializer(), kept) }.getOrNull()
    }

    /** Keyed by [user]'s own id, so a racing sign-out can only reach the row its purge deletes. */
    suspend fun remember(user: AuthUser) {
        val server = tokenStorage.getServerUrl() ?: return
        val scope = sessionScope(normalizedOrigin(server), user.id)
        runCatching { store.keepProfile(scope, json.encodeToString(AuthUser.serializer(), user)) }
            .onFailure { if (it is CancellationException) throw it }
    }

    /** Refetches on every reconnection until the server answers; null once [current] turns false. */
    suspend fun freshUser(
        current: () -> Boolean,
        fetch: suspend () -> Result<AuthUser>,
    ): AuthUser? {
        while (true) {
            val user =
                coroutineScope {
                    val reconnected =
                        async(start = CoroutineStart.UNDISPATCHED) { online.dropWhile { it }.first { it } }
                    val fetched = fetch().getOrNull()
                    if (fetched == null && current()) reconnected.await() else reconnected.cancel()
                    fetched
                }
            if (user != null || !current()) return user
        }
    }

    /** Runs inside a sign-in transition, once the user id is saved. */
    suspend fun register() {
        // A failed sweep must not strand sign-in; the scopes it missed stay purge_pending.
        runCatching { purger.purgeInactive(tokenStorage.currentOfflineScope()) }
            .onFailure { if (it is CancellationException) throw it }
        worker.resumeAuth()
    }

    suspend fun purge(scope: String) {
        purger.purge(scope)
    }

    private companion object {
        val json = Json { ignoreUnknownKeys = true }
    }
}
