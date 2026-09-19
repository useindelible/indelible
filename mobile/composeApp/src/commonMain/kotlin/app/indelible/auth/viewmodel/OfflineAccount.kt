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
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.dropWhile
import kotlinx.coroutines.flow.first
import kotlinx.serialization.json.Json

/**
 * The signed-in account's offline side: the profile kept for opening the app without the network,
 * and the scope housekeeping a sign-in or sign-out owes the offline store.
 */
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

    /**
     * Keeps [user] for the next start. The scope comes from the user's own id, so a sign-out that
     * races this can only reach the row of the account being signed out, which its purge deletes.
     */
    suspend fun remember(user: AuthUser) {
        val server = tokenStorage.getServerUrl() ?: return
        val scope = sessionScope(normalizedOrigin(server), user.id)
        // Losing the kept profile only costs the next start its offline entry.
        runCatching { store.keepProfile(scope, json.encodeToString(AuthUser.serializer(), user)) }
            .onFailure { if (it is CancellationException) throw it }
    }

    /**
     * The server's profile, fetched again each time connectivity returns until it answers or
     * [current] turns false. A confirmed rejection signs out through the transport, which ends it.
     */
    suspend fun freshUser(
        current: () -> Boolean,
        fetch: suspend () -> Result<AuthUser>,
    ): AuthUser? {
        while (true) {
            fetch().getOrNull()?.let { return it }
            online.dropWhile { it }.first { it }
            if (!current()) return null
        }
    }

    /** Runs inside a sign-in transition, once the user id is saved. */
    suspend fun register() {
        // Sweeping stale scopes is housekeeping, so a failure must not strand the session on the
        // splash: the scopes it could not clear keep purge_pending and are retried next launch.
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
