package app.indelible.auth.server

import io.ktor.client.HttpClient
import io.ktor.client.engine.HttpClientEngine
import io.ktor.client.request.get
import io.ktor.http.isSuccess

fun interface ServerHealthChecker {
    suspend fun check(baseUrl: String): Result<Unit>
}

class HttpServerHealthChecker(
    engine: HttpClientEngine? = null,
) : ServerHealthChecker {
    private val httpClient: HttpClient = engine?.let { HttpClient(it) } ?: HttpClient()

    override suspend fun check(baseUrl: String): Result<Unit> =
        runCatching {
            val response = httpClient.get(baseUrl.trimEnd('/') + "/api/health")
            check(response.status.isSuccess()) { "Server responded ${response.status}" }
        }
}
