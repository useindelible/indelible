package app.indelible.core.network

import app.indelible.api.generated.client.ApiConfiguration
import app.indelible.api.generated.client.ApiV1AuthRefreshClient
import app.indelible.api.generated.client.NetworkError
import app.indelible.api.generated.client.NetworkResult
import app.indelible.api.generated.models.RefreshResponse
import app.indelible.api.generated.models.RefreshTokenRequest
import app.indelible.core.model.ApiError
import app.indelible.core.platform.platformClientType
import app.indelible.core.storage.TokenStorage
import io.ktor.client.HttpClient
import io.ktor.client.engine.HttpClientEngine
import io.ktor.client.plugins.HttpTimeout
import io.ktor.client.plugins.contentnegotiation.ContentNegotiation
import io.ktor.client.plugins.defaultRequest
import io.ktor.client.request.header
import io.ktor.http.ContentType
import io.ktor.http.contentType
import io.ktor.serialization.kotlinx.json.json
import io.ktor.util.date.getTimeMillis
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.serialization.json.Json

class AuthenticatedApiTransport(
    private val tokenStorage: TokenStorage,
    private val onUnauthorized: suspend () -> Unit = {},
    engine: HttpClientEngine? = null,
) {
    private val jsonConfig =
        Json {
            ignoreUnknownKeys = true
            isLenient = true
            encodeDefaults = true
        }

    internal val httpClient: HttpClient =
        (engine?.let { HttpClient(it) } ?: HttpClient()).config {
            install(ContentNegotiation) {
                json(jsonConfig)
            }
            // No default values: ordinary requests keep the engine's timeouts.
            // Installed so long-lived calls (the Mila SSE stream) can widen
            // their own budget per request.
            install(HttpTimeout)
            defaultRequest {
                contentType(ContentType.Application.Json)
                header("X-Client-Type", platformClientType())
            }
        }

    private val refreshMutex = Mutex()

    internal suspend fun baseUrl(): String = tokenStorage.resolvedServerUrl()

    internal suspend fun <T> publicRequest(block: suspend (HttpClient, ApiConfiguration) -> NetworkResult<T>): Result<T> =
        runCatching {
            block(httpClient, ApiConfiguration(basePath = baseUrl())).getOrThrow()
        }

    internal suspend fun <T> authenticatedRequest(
        retryOn401: Boolean = true,
        block: suspend (HttpClient, ApiConfiguration) -> NetworkResult<T>,
    ): Result<T> =
        runCatching {
            authenticatedValue(retryOn401) { token ->
                block(httpClient, configuration(token)).getOrThrow()
            }
        }

    internal suspend fun <T> directAuthenticatedRequest(
        retryOn401: Boolean = true,
        block: suspend (HttpClient, String, String) -> T,
    ): Result<T> =
        runCatching {
            authenticatedValue(retryOn401) { token ->
                block(httpClient, baseUrl(), token)
            }
        }

    internal suspend fun bearerToken(): String = ensureValidToken()

    internal suspend fun refreshToken(): String? = tokenStorage.getRefreshToken()

    fun close() {
        httpClient.close()
    }

    suspend fun resolveImageRequest(url: String): ResolvedImageRequest {
        if (!url.contains("/api/v1/")) {
            return ResolvedImageRequest(url, bearerToken = null)
        }
        val reachableUrl = rewriteBackendOrigin(url)
        val token = runCatching { ensureValidToken() }.getOrNull()
        return ResolvedImageRequest(reachableUrl, bearerToken = token)
    }

    internal suspend fun rewriteBackendOrigin(url: String): String {
        val pathStart = url.indexOf("/api/")
        if (pathStart < 0) return url
        return baseUrl().trimEnd('/') + url.substring(pathStart)
    }

    private suspend fun configuration(token: String): ApiConfiguration =
        ApiConfiguration(
            basePath = baseUrl(),
            customHeaders = mapOf("Authorization" to "Bearer $token"),
        )

    private suspend fun <T> authenticatedValue(
        retryOn401: Boolean,
        block: suspend (token: String) -> T,
    ): T {
        val token = ensureValidToken()
        if (!retryOn401) return block(token)
        return try {
            block(token)
        } catch (error: ApiException) {
            if (error.statusCode != UNAUTHORIZED_STATUS) throw error
            try {
                block(refreshUnderLock(rejectedToken = token))
            } catch (retryError: ApiException) {
                if (retryError.statusCode == UNAUTHORIZED_STATUS) {
                    clearSession()
                }
                throw retryError
            }
        }
    }

    private suspend fun ensureValidToken(): String {
        val token = tokenStorage.getToken()
        val expiresAt = tokenStorage.getExpiresAt()
        val now = currentEpochSeconds()
        if (token != null && expiresAt != null && now < expiresAt - REFRESH_BUFFER_SECONDS) {
            return token
        }
        if (token != null && expiresAt == null) {
            return token
        }
        return refreshUnderLock()
    }

    private suspend fun refreshUnderLock(rejectedToken: String? = null): String =
        refreshMutex.withLock {
            val token = tokenStorage.getToken()
            val expiresAt = tokenStorage.getExpiresAt()
            val now = currentEpochSeconds()
            if (token != null && rejectedToken != null && token != rejectedToken) {
                return token
            }
            if (rejectedToken == null && token != null && expiresAt != null && now < expiresAt - REFRESH_BUFFER_SECONDS) {
                return token
            }

            val result = refreshTokens()
            if (result.isFailure) {
                clearSession()
                throw ApiException(UNAUTHORIZED_STATUS, "Session expired")
            }
            tokenStorage.getToken()
                ?: throw ApiException(UNAUTHORIZED_STATUS, "Token missing after refresh")
        }

    private suspend fun refreshTokens(): Result<RefreshResponse> {
        val refreshToken =
            tokenStorage.getRefreshToken()
                ?: return Result.failure(ApiException(UNAUTHORIZED_STATUS, "No refresh token"))
        return publicRequest { client, configuration ->
            ApiV1AuthRefreshClient(client).refresh(
                refreshTokenRequest = RefreshTokenRequest(refreshToken = refreshToken),
                apiConfiguration = configuration,
            )
        }.onSuccess { response ->
            tokenStorage.saveToken(response.accessToken)
            tokenStorage.saveExpiresAt(response.expiresAt)
            response.refreshToken?.let { tokenStorage.saveRefreshToken(it) }
        }
    }

    private suspend fun clearSession() {
        tokenStorage.clearAll()
        onUnauthorized()
    }

    companion object {
        const val DEFAULT_SERVER_URL = "http://localhost:38473"
        private const val UNAUTHORIZED_STATUS = 401
        private const val REFRESH_BUFFER_SECONDS = 120L
        private const val MS_PER_SECOND = 1000L
    }

    private fun currentEpochSeconds(): Long = getTimeMillis() / MS_PER_SECOND
}

internal fun <T> NetworkResult<T>.getOrThrow(): T =
    when (this) {
        is NetworkResult.Success -> data
        is NetworkResult.Failure -> throw error.asThrowable()
    }

internal fun NetworkError.asThrowable(): Throwable =
    when (this) {
        is NetworkError.Http -> ApiException(statusCode, apiErrorMessage(body, statusDescription))
        is NetworkError.Network -> cause ?: IllegalStateException("Network request failed")
        is NetworkError.Serialization -> cause
        is NetworkError.Unknown -> cause ?: IllegalStateException("API request failed")
    }

private fun apiErrorMessage(
    body: String?,
    fallback: String,
): String {
    if (body.isNullOrBlank()) return fallback
    return runCatching {
        apiErrorJson
            .decodeFromString<ApiError>(body)
            .let { it.message ?: it.error }
    }.getOrElse { body.take(ERROR_BODY_PREVIEW_CHARS) }
}

private const val ERROR_BODY_PREVIEW_CHARS = 200
private val apiErrorJson = Json { ignoreUnknownKeys = true }
