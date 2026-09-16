package app.indelible.core.offline

import app.indelible.api.generated.models.AppendReadingEventsBody
import app.indelible.api.generated.models.CreateHighlightBody
import app.indelible.api.generated.models.DocumentUpsertNoteBody
import app.indelible.api.generated.models.HighlightTagsBody
import app.indelible.api.generated.models.LocatorSchemaFlat
import app.indelible.api.generated.models.PatchHighlightBody
import app.indelible.api.generated.models.ReadingEventBody
import app.indelible.api.generated.models.ReadingPositionSchema
import app.indelible.api.generated.models.SourceLocatorSchemaFlat
import app.indelible.api.generated.models.UpsertNoteBody
import app.indelible.core.network.AuthenticatedApiTransport
import app.indelible.core.network.RawApiResponse
import io.ktor.client.request.HttpRequestBuilder
import io.ktor.client.request.delete
import io.ktor.client.request.header
import io.ktor.client.request.patch
import io.ktor.client.request.post
import io.ktor.client.request.put
import io.ktor.client.request.setBody
import io.ktor.http.HttpHeaders
import kotlinx.coroutines.CancellationException
import kotlinx.datetime.Instant
import kotlinx.serialization.json.Json

/** A batch is more than one row only when every row in it is READING_EVENT for the same document. */
interface OutboxSender {
    suspend fun send(
        scope: String,
        batch: List<OutboxRow>,
    ): SendOutcome
}

private const val REPLAYED_STATUS = 200
private const val NOT_FOUND_STATUS = 404
private val SUCCESS_STATUS_RANGE = 200..299

class ApiOutboxSender(
    private val store: OfflineStore,
    private val transport: AuthenticatedApiTransport,
) : OutboxSender {
    private val json = Json { ignoreUnknownKeys = true }

    override suspend fun send(
        scope: String,
        batch: List<OutboxRow>,
    ): SendOutcome {
        val result = runCatching { dispatch(scope, batch) }
        result.exceptionOrNull()?.let { if (it is CancellationException) throw it }
        return result.getOrElse { SendOutcome.Transport(it) }
    }

    private suspend fun dispatch(
        scope: String,
        batch: List<OutboxRow>,
    ): SendOutcome {
        val head = batch.first()
        return when (head.kind) {
            OutboxKind.READING_EVENT -> sendReadingEvents(scope, head.documentId, batch)
            OutboxKind.HIGHLIGHT_CREATE -> sendHighlightCreate(head)
            OutboxKind.HIGHLIGHT_COLOR -> sendHighlightColor(head)
            OutboxKind.HIGHLIGHT_NOTE -> sendHighlightNote(head)
            OutboxKind.HIGHLIGHT_TAGS -> sendHighlightTags(head)
            OutboxKind.HIGHLIGHT_DELETE -> sendHighlightDelete(head)
            OutboxKind.DOCUMENT_NOTE -> sendDocumentNote(head)
        }
    }

    private suspend fun sendReadingEvents(
        scope: String,
        documentId: String,
        batch: List<OutboxRow>,
    ): SendOutcome {
        val body =
            AppendReadingEventsBody(
                clientId = store.clientIdentity(scope).clientId,
                events = batch.map { readingEventBody(it.payload as OutboxPayload.ReadingEvent) },
            )
        val response =
            transport.rawAuthenticatedRequest { client, baseUrl, token ->
                client.post("$baseUrl/api/v1/documents/$documentId/reading-events") {
                    authorize(token)
                    setBody(body)
                }
            }
        return outcome(response)
    }

    private suspend fun sendHighlightCreate(row: OutboxRow): SendOutcome {
        val payload = row.payload as OutboxPayload.HighlightCreate
        val locatorJson =
            requireNotNull(payload.locatorJson) { "highlight ${payload.highlightId} was queued without a locator" }
        val body =
            CreateHighlightBody(
                color = payload.color,
                id = payload.highlightId,
                locator = json.decodeFromString<LocatorSchemaFlat>(locatorJson),
                sourceLocator = payload.sourceLocatorJson?.let { json.decodeFromString<SourceLocatorSchemaFlat>(it) },
                textContent = payload.textContent,
            )
        val response =
            transport.rawAuthenticatedRequest { client, baseUrl, token ->
                client.post("$baseUrl/api/v1/documents/${row.documentId}/highlights") {
                    authorize(token)
                    setBody(body)
                }
            }
        return outcome(response, replayStatus = REPLAYED_STATUS)
    }

    private suspend fun sendHighlightColor(row: OutboxRow): SendOutcome {
        val payload = row.payload as OutboxPayload.HighlightColor
        val response =
            transport.rawAuthenticatedRequest { client, baseUrl, token ->
                client.patch("$baseUrl/api/v1/highlights/${payload.highlightId}") {
                    authorize(token)
                    setBody(PatchHighlightBody(color = payload.color))
                }
            }
        return outcome(response)
    }

    private suspend fun sendHighlightNote(row: OutboxRow): SendOutcome {
        val payload = row.payload as OutboxPayload.HighlightNote
        val url = { baseUrl: String -> "$baseUrl/api/v1/highlights/${payload.highlightId}/note" }
        val noteBody = payload.body
        if (noteBody == null) {
            val response =
                transport.rawAuthenticatedRequest { client, baseUrl, token ->
                    client.delete(url(baseUrl)) { authorize(token) }
                }
            return outcome(response, replayStatus = NOT_FOUND_STATUS)
        }
        val response =
            transport.rawAuthenticatedRequest { client, baseUrl, token ->
                client.put(url(baseUrl)) {
                    authorize(token)
                    setBody(UpsertNoteBody(body = noteBody))
                }
            }
        return outcome(response)
    }

    private suspend fun sendHighlightTags(row: OutboxRow): SendOutcome {
        val payload = row.payload as OutboxPayload.HighlightTags
        val response =
            transport.rawAuthenticatedRequest { client, baseUrl, token ->
                client.put("$baseUrl/api/v1/highlights/${payload.highlightId}/tags") {
                    authorize(token)
                    setBody(HighlightTagsBody(tags = payload.tags))
                }
            }
        return outcome(response)
    }

    private suspend fun sendHighlightDelete(row: OutboxRow): SendOutcome {
        val payload = row.payload as OutboxPayload.HighlightDelete
        val response =
            transport.rawAuthenticatedRequest { client, baseUrl, token ->
                client.delete("$baseUrl/api/v1/highlights/${payload.highlightId}") { authorize(token) }
            }
        return outcome(response, replayStatus = NOT_FOUND_STATUS)
    }

    private suspend fun sendDocumentNote(row: OutboxRow): SendOutcome {
        val payload = row.payload as OutboxPayload.DocumentNote
        val response =
            transport.rawAuthenticatedRequest { client, baseUrl, token ->
                client.put("$baseUrl/api/v1/documents/${row.documentId}/note") {
                    authorize(token)
                    setBody(DocumentUpsertNoteBody(body = payload.body))
                }
            }
        return outcome(response)
    }

    private fun readingEventBody(payload: OutboxPayload.ReadingEvent): ReadingEventBody =
        ReadingEventBody(
            activeMs = payload.activeMs,
            assetKind = payload.assetKind,
            attempt = payload.attempt,
            cause = payload.cause,
            id = payload.eventId,
            kind = payload.kind,
            originSeq = payload.originSeq,
            position = payload.positionJson?.let { json.decodeFromString<ReadingPositionSchema>(it) },
            progressBasisPoints = payload.progressBasisPoints,
            recordedAt = Instant.fromEpochMilliseconds(payload.recordedAtEpochMs),
            sessionId = payload.sessionId,
        )

    private fun outcome(
        response: RawApiResponse,
        replayStatus: Int? = null,
    ): SendOutcome =
        when {
            response.status == replayStatus -> SendOutcome.ReplaySuccess
            response.status in SUCCESS_STATUS_RANGE -> SendOutcome.Success
            else -> SendOutcome.Http(response.status, response.retryAfterSeconds, response.bodyText)
        }
}

private fun HttpRequestBuilder.authorize(token: String) {
    header(HttpHeaders.Authorization, "Bearer $token")
}
