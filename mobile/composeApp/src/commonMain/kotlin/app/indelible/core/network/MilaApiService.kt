package app.indelible.core.network

import app.indelible.api.generated.client.ApiV1MilaConfigClient
import app.indelible.api.generated.client.ApiV1MilaConfigReindexClient
import app.indelible.api.generated.client.ApiV1MilaConfigTestClient
import app.indelible.api.generated.client.ApiV1MilaPresetsClient
import app.indelible.api.generated.client.ApiV1MilaSessionsClient
import app.indelible.api.generated.client.ApiV1MilaSessionsMessagesClient
import app.indelible.api.generated.models.CreateMilaPromptPresetBody
import app.indelible.api.generated.models.CreateMilaSessionBody
import app.indelible.api.generated.models.MilaConfigResponse
import app.indelible.api.generated.models.MilaConversationResponse
import app.indelible.api.generated.models.MilaPromptPresetResponse
import app.indelible.api.generated.models.MilaPromptPresetsResponse
import app.indelible.api.generated.models.MilaSessionListResponse
import app.indelible.api.generated.models.MilaSessionResponse
import app.indelible.api.generated.models.TestMilaConfigBody
import app.indelible.api.generated.models.TestMilaConfigResponse
import app.indelible.api.generated.models.UpdateMilaPromptPresetBody
import app.indelible.api.generated.models.UpsertMilaConfigBody
import io.ktor.client.plugins.timeout
import io.ktor.client.request.header
import io.ktor.client.request.parameter
import io.ktor.client.request.prepareGet
import io.ktor.client.statement.HttpResponse

class MilaApiService(
    private val transport: AuthenticatedApiTransport,
) {
    suspend fun getConfig(): Result<MilaConfigResponse> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1MilaConfigClient(client).getConfig(configuration)
        }

    suspend fun upsertConfig(body: UpsertMilaConfigBody): Result<MilaConfigResponse> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1MilaConfigClient(client).upsertConfig(body, configuration)
        }

    suspend fun reindexConfig(body: UpsertMilaConfigBody): Result<MilaConfigResponse> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1MilaConfigReindexClient(client).reindexConfig(body, configuration)
        }

    suspend fun testConfig(body: TestMilaConfigBody): Result<TestMilaConfigResponse> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1MilaConfigTestClient(client).testConfig(body, configuration)
        }

    suspend fun getPromptPresets(): Result<MilaPromptPresetsResponse> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1MilaPresetsClient(client).listPromptPresets(configuration)
        }

    suspend fun createPromptPreset(body: CreateMilaPromptPresetBody): Result<MilaPromptPresetResponse> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1MilaPresetsClient(client).createPromptPreset(body, configuration)
        }

    suspend fun updatePromptPreset(
        presetId: String,
        body: UpdateMilaPromptPresetBody,
    ): Result<MilaPromptPresetResponse> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1MilaPresetsClient(client).updatePromptPreset(body, presetId, configuration)
        }

    suspend fun deletePromptPreset(presetId: String): Result<Unit> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1MilaPresetsClient(client).deletePromptPreset(presetId, configuration)
        }

    suspend fun listSessions(limit: Int = 50): Result<MilaSessionListResponse> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1MilaSessionsClient(client).listSessions(limit.toLong(), configuration)
        }

    suspend fun createSession(body: CreateMilaSessionBody): Result<MilaSessionResponse> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1MilaSessionsClient(client).createSession(body, configuration)
        }

    suspend fun getMessages(sessionId: String): Result<MilaConversationResponse> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1MilaSessionsMessagesClient(client).getSessionMessages(sessionId, configuration)
        }

    /**
     * Opens the Mila SSE stream and hands the live response to [block] inside a
     * prepared-statement execute scope. Ktor 3 saves plain request bodies to
     * memory before returning them, which would hold every delta until the
     * answer finished; only inside `prepareGet(...).execute` does the body
     * channel deliver bytes as they arrive.
     */
    internal suspend fun <T> withChatStream(
        sessionId: String,
        question: String,
        block: suspend (HttpResponse) -> T,
    ): Result<T> =
        transport.directAuthenticatedRequest { client, baseUrl, token ->
            client
                .prepareGet("$baseUrl/api/v1/mila/stream") {
                    header("Authorization", "Bearer $token")
                    parameter("session_id", sessionId)
                    parameter("question", question)
                    // An LLM answer streams for minutes and can pause between
                    // tokens; the engine's default read timeout kills it mid-reply.
                    timeout {
                        requestTimeoutMillis = STREAM_REQUEST_TIMEOUT_MS
                        socketTimeoutMillis = STREAM_SOCKET_TIMEOUT_MS
                    }
                }.execute { response -> block(response) }
        }

    private companion object {
        const val STREAM_REQUEST_TIMEOUT_MS = 10 * 60 * 1000L
        const val STREAM_SOCKET_TIMEOUT_MS = 5 * 60 * 1000L
    }
}
