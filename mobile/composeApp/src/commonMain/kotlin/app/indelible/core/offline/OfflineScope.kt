package app.indelible.core.offline

import app.indelible.core.network.normalizedOrigin
import app.indelible.core.storage.TokenStorage

/**
 * Identifies the isolated offline dataset for the account currently connected. Unresolved
 * (no stored server, or no cached user id yet) until [app.indelible.auth.viewmodel.AuthViewModel]
 * has completed a session fetch, so callers must treat null as "offline store unavailable" rather
 * than falling back to a default server.
 */
suspend fun TokenStorage.currentOfflineScope(): String? =
    getServerUrl()?.let { serverUrl ->
        getUserId()?.let { userId -> "${normalizedOrigin(serverUrl)}|$userId" }
    }
