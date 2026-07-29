package app.indelible.core.storage

import app.indelible.auth.oauth.PendingOAuthFlow

interface TokenStorage {
    suspend fun saveToken(token: String)

    suspend fun getToken(): String?

    suspend fun clearToken()

    suspend fun saveServerUrl(url: String)

    suspend fun getServerUrl(): String?

    suspend fun saveRefreshToken(token: String)

    suspend fun getRefreshToken(): String?

    suspend fun saveExpiresAt(epochSeconds: Long)

    suspend fun getExpiresAt(): Long?

    suspend fun savePendingOAuthFlow(flow: PendingOAuthFlow)

    suspend fun getPendingOAuthFlow(): PendingOAuthFlow?

    suspend fun clearPendingOAuthFlow()

    suspend fun clearAll() {
        clearToken()
        clearPendingOAuthFlow()
    }
}
