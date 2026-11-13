package app.indelible.core.offline

import app.indelible.core.network.AuthenticatedApiTransport
import app.indelible.core.network.ServerCapabilities
import app.indelible.core.storage.InMemoryTokenStorage
import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.MockRequestHandleScope
import io.ktor.client.request.HttpRequestData
import io.ktor.client.request.HttpResponseData
import io.ktor.http.HttpHeaders
import io.ktor.http.HttpMethod
import io.ktor.http.content.TextContent
import io.ktor.http.headersOf

internal const val SENDER_ORIGIN = "http://localhost:38473"
internal const val SENDER_SCOPE = "$SENDER_ORIGIN|usr_1"
internal const val SENDER_DOCUMENT = "doc_01"
internal const val SENDER_HIGHLIGHT = "hlt_01"
internal const val SENDER_CLIENT_ID = "cli_test"
internal const val SENDER_TOKEN_EXPIRY = 4_102_444_800L
internal val SENDER_SESSION = Session(epoch = 0, origin = SENDER_ORIGIN, scope = SENDER_SCOPE)
internal val senderJsonHeaders = headersOf(HttpHeaders.ContentType, "application/json")

internal class SenderCapture {
    val methods = mutableListOf<HttpMethod>()
    val paths = mutableListOf<String>()
    val bodies = mutableListOf<String?>()
}

internal suspend fun boundSenderRegistry(): SessionRegistry {
    val registry = SessionRegistry()
    registry.publish(SessionState(0, SENDER_SESSION))
    return registry
}

internal fun senderOver(transport: AuthenticatedApiTransport) = ApiOutboxSender(transport, ServerCapabilities(transport, clock = { 0L }))

/** A sender over a mock engine with a valid token, so every request reaches [handler] with one authorization. */
internal suspend fun senderWith(
    capture: SenderCapture = SenderCapture(),
    handler: MockRequestHandleScope.(HttpRequestData) -> HttpResponseData,
): ApiOutboxSender {
    val tokenStorage = InMemoryTokenStorage()
    tokenStorage.saveToken("test-token")
    tokenStorage.saveExpiresAt(SENDER_TOKEN_EXPIRY)
    val engine =
        MockEngine { request ->
            capture.methods += request.method
            capture.paths += request.url.encodedPath
            capture.bodies += (request.body as? TextContent)?.text
            handler(request)
        }
    return senderOver(AuthenticatedApiTransport(tokenStorage, engine = engine, registry = boundSenderRegistry()))
}

internal fun senderRow(
    kind: OutboxKind,
    payload: OutboxPayload,
    entityId: String = SENDER_HIGHLIGHT,
) = OutboxRow(
    seq = 1,
    scope = SENDER_SCOPE,
    id = "obx_1",
    kind = kind,
    entityId = entityId,
    documentId = SENDER_DOCUMENT,
    payload = payload,
    createdAt = 0,
    attempts = 0,
    lastAttemptAt = null,
    nextAttemptAt = 0,
    state = OutboxState.PENDING,
    lastError = null,
)
