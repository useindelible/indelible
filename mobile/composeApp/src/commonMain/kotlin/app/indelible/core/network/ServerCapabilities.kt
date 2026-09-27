package app.indelible.core.network

import app.indelible.core.offline.Session
import io.ktor.client.plugins.timeout
import io.ktor.client.request.get
import io.ktor.client.statement.bodyAsText
import io.ktor.http.HttpStatusCode
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive

const val EPUB_CHAPTER_LIST = "epub_chapter_list"

/**
 * Answers whether the signed-in server offers a feature: `true` when listed, `false` when the
 * server answers without it (or predates the endpoint), `null` when it could not be asked. A yes
 * holds for the session; a no is re-asked after five minutes so a server upgrade is noticed.
 */
class ServerCapabilities(
    private val transport: AuthenticatedApiTransport,
    private val clock: () -> Long,
) {
    private class Answer(
        val supported: Boolean,
        val at: Long,
    )

    private val lock = Mutex()
    private var owner: Session? = null
    private val answers = mutableMapOf<String, Answer>()

    suspend fun supports(
        session: Session,
        capability: String,
    ): Boolean? = cached(session, capability) ?: ask(session, capability)?.also { remember(session, capability, it) }

    private suspend fun remember(
        session: Session,
        capability: String,
        supported: Boolean,
    ) {
        lock.withLock {
            if (owner === session) answers[capability] = Answer(supported, clock())
        }
    }

    private suspend fun cached(
        session: Session,
        capability: String,
    ): Boolean? =
        lock.withLock {
            if (owner !== session) {
                owner = session
                answers.clear()
            }
            val answer = answers[capability] ?: return null
            when {
                answer.supported -> true
                clock() - answer.at < NEGATIVE_TTL_MS -> false
                else -> null
            }
        }

    private suspend fun ask(
        session: Session,
        capability: String,
    ): Boolean? {
        val result =
            runCatching {
                val response =
                    transport.httpClient.get(session.origin + CAPABILITIES_PATH) {
                        timeout { requestTimeoutMillis = REQUEST_TIMEOUT_MS }
                    }
                when (response.status) {
                    HttpStatusCode.OK -> listed(response.bodyAsText(), capability)
                    HttpStatusCode.NotFound -> false
                    else -> null
                }
            }
        result.exceptionOrNull()?.let { if (it is CancellationException) throw it }
        return result.getOrNull()
    }

    // A server that predates the route serves the web app's index.html for unknown paths with a 200,
    // so any body that is not the capabilities object means the route is absent.
    private fun listed(
        body: String,
        capability: String,
    ): Boolean {
        val element = runCatching { Json.parseToJsonElement(body) }.getOrNull()
        val capabilities = (element as? JsonObject)?.get("capabilities") as? JsonArray
        return capabilities?.contains(JsonPrimitive(capability)) == true
    }

    private companion object {
        const val CAPABILITIES_PATH = "/api/v1/server/capabilities"
        const val NEGATIVE_TTL_MS = 5 * 60 * 1000L
        const val REQUEST_TIMEOUT_MS = 10_000L
    }
}
