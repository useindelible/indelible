package app.indelible.core.storage

import android.content.Context
import android.content.SharedPreferences
import androidx.security.crypto.EncryptedSharedPreferences
import androidx.security.crypto.MasterKeys
import app.indelible.auth.oauth.PendingOAuthFlow
import kotlinx.serialization.json.Json

class AndroidTokenStorage(
    context: Context,
) : TokenStorage {
    private val prefs: SharedPreferences by lazy {
        val masterKeyAlias = MasterKeys.getOrCreate(MasterKeys.AES256_GCM_SPEC)
        EncryptedSharedPreferences.create(
            PREFS_NAME,
            masterKeyAlias,
            context,
            EncryptedSharedPreferences.PrefKeyEncryptionScheme.AES256_SIV,
            EncryptedSharedPreferences.PrefValueEncryptionScheme.AES256_GCM,
        )
    }

    override suspend fun saveToken(token: String) {
        prefs.edit().putString(KEY_TOKEN, token).commit()
    }

    override suspend fun getToken(): String? = prefs.getString(KEY_TOKEN, null)

    override suspend fun clearToken() {
        prefs.edit().remove(KEY_TOKEN).commit()
    }

    override suspend fun saveServerUrl(url: String) {
        prefs.edit().putString(KEY_SERVER_URL, url).commit()
    }

    override suspend fun getServerUrl(): String? = prefs.getString(KEY_SERVER_URL, null)

    override suspend fun saveRefreshToken(token: String) {
        prefs.edit().putString(KEY_REFRESH_TOKEN, token).commit()
    }

    override suspend fun getRefreshToken(): String? = prefs.getString(KEY_REFRESH_TOKEN, null)

    override suspend fun saveExpiresAt(epochSeconds: Long) {
        prefs.edit().putLong(KEY_EXPIRES_AT, epochSeconds).commit()
    }

    override suspend fun getExpiresAt(): Long? =
        if (prefs.contains(KEY_EXPIRES_AT)) prefs.getLong(KEY_EXPIRES_AT, 0L) else null

    override suspend fun savePendingOAuthFlow(flow: PendingOAuthFlow) {
        prefs.edit().putString(KEY_PENDING_OAUTH, Json.encodeToString(flow)).commit()
    }

    override suspend fun getPendingOAuthFlow(): PendingOAuthFlow? =
        prefs.getString(KEY_PENDING_OAUTH, null)?.let {
            runCatching { Json.decodeFromString<PendingOAuthFlow>(it) }.getOrNull()
        }

    override suspend fun clearPendingOAuthFlow() {
        prefs.edit().remove(KEY_PENDING_OAUTH).commit()
    }

    override suspend fun clearAll() {
        prefs
            .edit()
            .remove(KEY_TOKEN)
            .remove(KEY_REFRESH_TOKEN)
            .remove(KEY_EXPIRES_AT)
            .remove(KEY_PENDING_OAUTH)
            .remove(KEY_PENDING_ITEMS)
            .commit()
    }

    companion object {
        private const val PREFS_NAME = "indelible_auth"
        private const val KEY_TOKEN = "auth_token"
        private const val KEY_REFRESH_TOKEN = "refresh_token"
        private const val KEY_EXPIRES_AT = "expires_at"
        private const val KEY_SERVER_URL = "server_url"
        private const val KEY_PENDING_OAUTH = "pending_oauth_flow"
        private const val KEY_PENDING_ITEMS = "pending_items"
    }
}
