package app.indelible.core.storage

import app.indelible.auth.oauth.PendingOAuthFlow

class InMemoryTokenStorage : TokenStorage {
    private var token: String? = null
    private var refreshToken: String? = null
    private var expiresAt: Long? = null
    private var serverUrl: String? = null
    private var pendingOAuthFlow: PendingOAuthFlow? = null
    private var pendingItems: String? = null

    override suspend fun saveToken(token: String) {
        this.token = token
    }

    override suspend fun getToken(): String? = token

    override suspend fun clearToken() {
        token = null
    }

    override suspend fun saveServerUrl(url: String) {
        serverUrl = url
    }

    override suspend fun getServerUrl(): String? = serverUrl

    override suspend fun saveRefreshToken(token: String) {
        refreshToken = token
    }

    override suspend fun getRefreshToken(): String? = refreshToken

    override suspend fun saveExpiresAt(epochSeconds: Long) {
        expiresAt = epochSeconds
    }

    override suspend fun getExpiresAt(): Long? = expiresAt

    override suspend fun savePendingOAuthFlow(flow: PendingOAuthFlow) {
        pendingOAuthFlow = flow
    }

    override suspend fun getPendingOAuthFlow(): PendingOAuthFlow? = pendingOAuthFlow

    override suspend fun clearPendingOAuthFlow() {
        pendingOAuthFlow = null
    }

    override suspend fun clearAll() {
        token = null
        refreshToken = null
        expiresAt = null
        pendingOAuthFlow = null
        pendingItems = null
    }

    suspend fun savePendingItems(raw: String) {
        pendingItems = raw
    }

    suspend fun getPendingItems(): String? = pendingItems
}
