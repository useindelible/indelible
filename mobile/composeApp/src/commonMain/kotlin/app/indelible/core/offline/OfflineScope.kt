package app.indelible.core.offline

import app.indelible.core.network.normalizedOrigin
import app.indelible.core.storage.TokenStorage

/** The connected account's offline dataset; null until a session fetch cached the user id. */
suspend fun TokenStorage.currentOfflineScope(): String? =
    getServerUrl()?.let { serverUrl ->
        getUserId()?.let { userId -> sessionScope(normalizedOrigin(serverUrl), userId) }
    }

internal fun sessionScope(
    origin: String,
    userId: String,
): String = "$origin|$userId"
