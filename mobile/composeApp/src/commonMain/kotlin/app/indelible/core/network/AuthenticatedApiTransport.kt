package app.indelible.core.network

import app.indelible.api.generated.client.ApiConfiguration
import app.indelible.api.generated.client.ApiV1AuthRefreshClient
import app.indelible.api.generated.client.NetworkError
import app.indelible.api.generated.client.NetworkResult
import app.indelible.api.generated.models.RefreshResponse
import app.indelible.api.generated.models.RefreshTokenRequest
import app.indelible.core.model.ApiError
import app.indelible.core.offline.Session
import app.indelible.core.offline.SessionRegistry
import app.indelible.core.offline.StaleSessionException
import app.indelible.core.platform.platformClientType
import app.indelible.core.storage.TokenStorage
import io.ktor.client.HttpClient
import io.ktor.client.engine.HttpClientEngine
import io.ktor.client.plugins.HttpTimeout
import io.ktor.client.plugins.contentnegotiation.ContentNegotiation
import io.ktor.client.plugins.defaultRequest
import io.ktor.client.request.header
import io.ktor.client.statement.HttpResponse
import io.ktor.client.statement.bodyAsText
import io.ktor.http.ContentType
import io.ktor.http.HttpHeaders
import io.ktor.http.contentType
import io.ktor.http.fromHttpToGmtDate
import io.ktor.http.isSuccess
import io.ktor.serialization.kotlinx.json.json
import io.ktor.util.date.getTimeMillis
import kotlinx.coroutines.channels.BufferOverflow
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.serialization.json.Json

/**
 * A response the caller inspects by status instead of by thrown failure. [bodyText] is empty on
 * success so a caller that only needs the status never pays for the body.
 */
data class RawApiResponse(
    val status: Int,
    val retryAfterSeconds: Long?,
    val bodyText: String,
)

class AuthenticatedApiTransport(
    private val tokenStorage: TokenStorage,
    private val onUnauthorized: suspend (epoch: Long) -> Unit = {},
    engine: HttpClientEngine? = null,
    private val registry: SessionRegistry = SessionRegistry(),
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

    private val requestSucceededState =
        MutableSharedFlow<Unit>(extraBufferCapacity = 1, onBufferOverflow = BufferOverflow.DROP_OLDEST)

    /** Emits once per call that reached the server and came back 2xx. */
    val requestSucceeded: SharedFlow<Unit> = requestSucceededState.asSharedFlow()

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
            val origin = baseUrl()
            authenticatedValue(retryOn401, origin) { token ->
                block(httpClient, configuration(origin, token)).getOrThrow()
            }
        }.onSuccess { requestSucceededState.tryEmit(Unit) }

    internal suspend fun <T> directAuthenticatedRequest(
        retryOn401: Boolean = true,
        block: suspend (HttpClient, String, String) -> T,
    ): Result<T> =
        runCatching {
            val origin = baseUrl()
            authenticatedValue(retryOn401, origin) { token ->
                block(httpClient, origin, token)
            }
        }.onSuccess { requestSucceededState.tryEmit(Unit) }

    /**
     * Surfaces the status instead of throwing on it, for callers that must classify 429/5xx
     * themselves. A block that simply returned a 401 would bypass [authenticatedValue]'s refresh,
     * so the 401 is rethrown inside the block to engage it; a 401 that reaches the caller here has
     * therefore already survived a refresh and means the session, not the request, is rejected.
     * The request is bound to [session]: it and any refresh it needs go to that session's origin,
     * and it is refused with [StaleSessionException] if the session was withdrawn before the first
     * send or before the retry that follows a refresh. A request already on the wire returns its
     * real outcome.
     */
    suspend fun rawAuthenticatedRequest(
        session: Session,
        block: suspend (client: HttpClient, baseUrl: String, token: String) -> HttpResponse,
    ): RawApiResponse =
        try {
            if (registry.current.value.session !== session) throw StaleSessionException()
            authenticatedValue(retryOn401 = true, session.origin) { token ->
                if (registry.current.value.session !== session) throw StaleSessionException()
                val response = block(httpClient, session.origin, token)
                if (response.status.value == UNAUTHORIZED_STATUS) {
                    throw ApiException(UNAUTHORIZED_STATUS, response.bodyAsText())
                }
                RawApiResponse(
                    status = response.status.value,
                    retryAfterSeconds = response.headers[HttpHeaders.RetryAfter]?.let(::retryAfterSeconds),
                    bodyText = if (response.status.isSuccess()) "" else response.bodyAsText(),
                )
            }.also { if (it.status in SUCCESS_STATUS_RANGE) requestSucceededState.tryEmit(Unit) }
        } catch (error: ApiException) {
            RawApiResponse(error.statusCode, retryAfterSeconds = null, bodyText = error.message)
        }

    internal suspend fun bearerToken(): String = ensureValidToken(baseUrl(), registry.current.value.epoch)

    internal suspend fun refreshToken(): String? = tokenStorage.getRefreshToken()

    fun close() {
        httpClient.close()
    }

    suspend fun resolveImageRequest(url: String): ResolvedImageRequest {
        if (!url.contains("/api/v1/")) {
            return ResolvedImageRequest(url, bearerToken = null)
        }
        val reachableUrl = rewriteBackendOrigin(url)
        val token = runCatching { ensureValidToken(baseUrl(), registry.current.value.epoch) }.getOrNull()
        return ResolvedImageRequest(reachableUrl, bearerToken = token)
    }

    internal suspend fun rewriteBackendOrigin(url: String): String {
        val pathStart = url.indexOf("/api/")
        if (pathStart < 0) return url
        return baseUrl().trimEnd('/') + url.substring(pathStart)
    }

    private fun configuration(
        origin: String,
        token: String,
    ): ApiConfiguration =
        ApiConfiguration(
            basePath = origin,
            customHeaders = mapOf("Authorization" to "Bearer $token"),
        )

    private suspend fun <T> authenticatedValue(
        retryOn401: Boolean,
        origin: String,
        block: suspend (token: String) -> T,
    ): T {
        // The retry may only use a token minted under the epoch the request started in: after a
        // transition the stored token belongs to another account and must not reach this origin.
        val epoch = registry.current.value.epoch
        val token = ensureValidToken(origin, epoch)
        if (!retryOn401) return block(token)
        return try {
            block(token)
        } catch (error: ApiException) {
            if (error.statusCode != UNAUTHORIZED_STATUS) throw error
            val replacement = refreshUnderLock(origin, epoch, rejectedToken = token)
            requireEpoch(epoch)
            try {
                block(replacement)
            } catch (retryError: ApiException) {
                if (retryError.statusCode == UNAUTHORIZED_STATUS) {
                    clearSession(epoch)
                }
                throw retryError
            }
        }
    }

    private fun requireEpoch(epoch: Long) {
        if (registry.current.value.epoch != epoch) throw StaleSessionException()
    }

    private suspend fun ensureValidToken(
        origin: String,
        epoch: Long,
    ): String {
        val token = tokenStorage.getToken()
        val expiresAt = tokenStorage.getExpiresAt()
        val now = currentEpochSeconds()
        if (token != null && expiresAt != null && now < expiresAt - REFRESH_BUFFER_SECONDS) {
            return token
        }
        if (token != null && expiresAt == null) {
            return token
        }
        return refreshUnderLock(origin, epoch)
    }

    // The epoch is the caller's, checked again once the lock is held: a request that queued behind
    // another refresh across a transition must not read, send or accept the next account's tokens.
    private suspend fun refreshUnderLock(
        origin: String,
        epoch: Long,
        rejectedToken: String? = null,
    ): String =
        refreshMutex.withLock {
            requireEpoch(epoch)
            val token = tokenStorage.getToken()
            val expiresAt = tokenStorage.getExpiresAt()
            val now = currentEpochSeconds()
            if (token != null && rejectedToken != null && token != rejectedToken) {
                return token
            }
            if (rejectedToken == null && token != null && expiresAt != null && now < expiresAt - REFRESH_BUFFER_SECONDS) {
                return token
            }

            val failure = refreshTokens(origin, epoch).exceptionOrNull()
            if (failure != null) {
                if (failure is ApiException && failure.statusCode in SESSION_REJECTED_STATUSES) {
                    clearSession(epoch)
                    throw ApiException(UNAUTHORIZED_STATUS, "Session expired")
                }
                throw failure
            }
            tokenStorage.getToken()
                ?: throw ApiException(UNAUTHORIZED_STATUS, "Token missing after refresh")
        }

    // The refresh binds to the epoch it started under and to the caller's origin: a result that
    // lands after or during a transition is discarded rather than written over the newer session's
    // credentials, and a refresh never starts while one is open, so the refresh token is never
    // posted to a server URL the transition is in the middle of replacing.
    private suspend fun refreshTokens(
        origin: String,
        epoch: Long,
    ): Result<RefreshResponse> {
        val refreshToken = tokenStorage.getRefreshToken()
        val state = registry.current.value
        if (refreshToken == null || state.transitioning || state.epoch != epoch) {
            val refusal =
                if (refreshToken == null) {
                    ApiException(UNAUTHORIZED_STATUS, "No refresh token")
                } else {
                    StaleSessionException()
                }
            return Result.failure(refusal)
        }
        return runCatching {
            ApiV1AuthRefreshClient(httpClient)
                .refresh(
                    refreshTokenRequest = RefreshTokenRequest(refreshToken = refreshToken),
                    apiConfiguration = ApiConfiguration(basePath = origin),
                ).getOrThrow()
        }.mapCatching { response ->
            val accepted =
                registry.publishRefreshed(epoch) {
                    tokenStorage.saveToken(response.accessToken)
                    tokenStorage.saveExpiresAt(response.expiresAt)
                    response.refreshToken?.let { tokenStorage.saveRefreshToken(it) }
                }
            if (!accepted) throw StaleSessionException()
            response
        }
    }

    /** Withdraws the session for the epoch that failed; clearing storage is the sign-out's job. */
    private suspend fun clearSession(epoch: Long) {
        if (registry.invalidate(epoch)) onUnauthorized(epoch)
    }

    companion object {
        const val DEFAULT_SERVER_URL = "http://localhost:38473"
        private const val UNAUTHORIZED_STATUS = 401
        private val SESSION_REJECTED_STATUSES = setOf(401, 403)
        private val SUCCESS_STATUS_RANGE = 200..299
        private const val REFRESH_BUFFER_SECONDS = 120L
        private const val MS_PER_SECOND = 1000L
    }

    private fun currentEpochSeconds(): Long = getTimeMillis() / MS_PER_SECOND

    // Retry-After is either delay-seconds or an HTTP-date; a date already in the past means retry now.
    private fun retryAfterSeconds(header: String): Long? =
        header.trim().toLongOrNull()?.takeIf { it >= 0 }
            ?: runCatching { header.trim().fromHttpToGmtDate() }.getOrNull()?.let { date ->
                ((date.timestamp - getTimeMillis()) / MS_PER_SECOND).coerceAtLeast(0)
            }
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
