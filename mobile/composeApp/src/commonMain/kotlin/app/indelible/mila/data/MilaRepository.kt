package app.indelible.mila.data

import app.indelible.api.generated.models.CreateMilaSessionBody
import app.indelible.api.generated.models.MilaConfigResponse
import app.indelible.api.generated.models.MilaConversationResponse
import app.indelible.api.generated.models.MilaSessionListResponse
import app.indelible.api.generated.models.MilaSessionResponse
import app.indelible.core.network.MilaApiService
import io.ktor.client.statement.bodyAsChannel
import io.ktor.http.isSuccess
import io.ktor.utils.io.readUTF8Line
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flow
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json

open class MilaRepository(
    private val milaApiService: MilaApiService,
) {
    open suspend fun getMilaConfig(): Result<MilaConfigResponse> = milaApiService.getConfig()

    open suspend fun listSessions(limit: Int = 50): Result<MilaSessionListResponse> = milaApiService.listSessions(limit)

    open suspend fun createSession(
        sessionType: String,
        documentId: String? = null,
        collectionId: String? = null,
    ): Result<MilaSessionResponse> =
        milaApiService.createSession(
            CreateMilaSessionBody(
                sessionType = sessionType,
                documentId = documentId,
                collectionId = collectionId,
            ),
        )

    open suspend fun getMessages(sessionId: String): Result<MilaConversationResponse> = milaApiService.getMessages(sessionId)

    open fun streamChat(
        sessionId: String,
        question: String,
    ): Flow<StreamEvent> =
        flow {
            // The whole read happens inside the prepared execute scope so the
            // body channel streams live; the same coroutine runs the flow
            // builder and the block, so emitting from inside is sound.
            val outcome =
                milaApiService.withChatStream(sessionId, question) { response ->
                    if (!response.status.isSuccess()) {
                        emit(StreamEvent.Error("Request failed: ${response.status.value}"))
                        return@withChatStream
                    }
                    val channel = response.bodyAsChannel()
                    while (!channel.isClosedForRead) {
                        val line = channel.readUTF8Line() ?: break
                        when (val parsed = parseSseLine(line)) {
                            SseLine.Ignore -> Unit
                            SseLine.Done -> {
                                emit(StreamEvent.Done)
                                return@withChatStream
                            }
                            is SseLine.Error -> {
                                emit(StreamEvent.Error(parsed.message))
                                return@withChatStream
                            }
                            is SseLine.Delta -> emit(StreamEvent.Delta(parsed.text))
                        }
                    }
                }
            // Transport failures (open or mid-read) propagate to the
            // collector, whose catch turns them into in-chat errors.
            outcome.getOrThrow()
        }
}

private val sseJson = Json { ignoreUnknownKeys = true }

internal sealed interface SseLine {
    data object Ignore : SseLine

    data object Done : SseLine

    data class Error(
        val message: String,
    ) : SseLine

    data class Delta(
        val text: String,
    ) : SseLine
}

private inline fun <reified T> decodeSse(data: String): T? = runCatching { sseJson.decodeFromString<T>(data) }.getOrNull()

/**
 * Classifies a single Mila SSE stream line into a [SseLine] event. Non-data lines,
 * blank payloads, and unparseable frames collapse to [SseLine.Ignore]; `[DONE]`,
 * error frames, and delta frames map to their typed events.
 */
internal fun parseSseLine(line: String): SseLine {
    if (!line.startsWith("data:")) return SseLine.Ignore
    val data = line.removePrefix("data:").trim()
    return when {
        data.isEmpty() -> SseLine.Ignore
        data == "[DONE]" -> SseLine.Done
        data.startsWith("{\"error\"") ->
            SseLine.Error(decodeSse<SseErrorPayload>(data)?.error ?: "Unknown stream error")
        data.startsWith("{\"delta\"") ->
            decodeSse<SseDeltaPayload>(data)?.let { SseLine.Delta(it.delta) } ?: SseLine.Ignore
        else -> SseLine.Ignore
    }
}

@Serializable
private data class SseDeltaPayload(
    @SerialName("delta") val delta: String,
)

@Serializable
private data class SseErrorPayload(
    @SerialName("error") val error: String,
)
