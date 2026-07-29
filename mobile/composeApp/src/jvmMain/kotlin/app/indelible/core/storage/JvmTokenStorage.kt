package app.indelible.core.storage

import app.indelible.auth.oauth.PendingOAuthFlow
import kotlinx.serialization.json.Json
import java.util.prefs.Preferences

class JvmTokenStorage : TokenStorage {
    private val prefs = Preferences.userNodeForPackage(JvmTokenStorage::class.java)

    override suspend fun saveToken(token: String) {
        prefs.put(KEY_TOKEN, token)
        prefs.flush()
    }

    override suspend fun getToken(): String? = prefs.get(KEY_TOKEN, null)

    override suspend fun clearToken() {
        prefs.remove(KEY_TOKEN)
        prefs.flush()
    }

    override suspend fun saveServerUrl(url: String) {
        prefs.put(KEY_SERVER_URL, url)
        prefs.flush()
    }

    override suspend fun getServerUrl(): String? = prefs.get(KEY_SERVER_URL, null)

    override suspend fun saveRefreshToken(token: String) {
        prefs.put(KEY_REFRESH_TOKEN, token)
        prefs.flush()
    }

    override suspend fun getRefreshToken(): String? = prefs.get(KEY_REFRESH_TOKEN, null)

    override suspend fun saveExpiresAt(epochSeconds: Long) {
        prefs.putLong(KEY_EXPIRES_AT, epochSeconds)
        prefs.flush()
    }

    override suspend fun getExpiresAt(): Long? =
        if (prefs.get(KEY_EXPIRES_AT, null) != null) prefs.getLong(KEY_EXPIRES_AT, 0L) else null

    override suspend fun savePendingOAuthFlow(flow: PendingOAuthFlow) {
        prefs.put(KEY_PENDING_OAUTH, Json.encodeToString(flow))
        prefs.flush()
    }

    override suspend fun getPendingOAuthFlow(): PendingOAuthFlow? =
        prefs.get(KEY_PENDING_OAUTH, null)?.let {
            runCatching { Json.decodeFromString<PendingOAuthFlow>(it) }.getOrNull()
        }

    override suspend fun clearPendingOAuthFlow() {
        prefs.remove(KEY_PENDING_OAUTH)
        prefs.flush()
    }

    override suspend fun clearAll() {
        prefs.remove(KEY_TOKEN)
        prefs.remove(KEY_REFRESH_TOKEN)
        prefs.remove(KEY_EXPIRES_AT)
        prefs.remove(KEY_PENDING_OAUTH)
        prefs.remove(KEY_PENDING_ITEMS)
        prefs.flush()
    }

    companion object {
        private const val KEY_TOKEN = "auth_token"
        private const val KEY_REFRESH_TOKEN = "refresh_token"
        private const val KEY_EXPIRES_AT = "expires_at"
        private const val KEY_SERVER_URL = "server_url"
        private const val KEY_PENDING_OAUTH = "pending_oauth_flow"
        private const val KEY_PENDING_ITEMS = "pending_items"
    }
}
