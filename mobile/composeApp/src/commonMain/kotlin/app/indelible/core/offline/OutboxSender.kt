package app.indelible.core.offline

import app.indelible.api.generated.models.AppendReadingEventsBody
import app.indelible.api.generated.models.CreateHighlightBody
import app.indelible.api.generated.models.DocumentNoteConflictResponse
import app.indelible.api.generated.models.DocumentNoteResponse
import app.indelible.api.generated.models.DocumentUpsertNoteBody
import app.indelible.api.generated.models.HighlightTagsBody
import app.indelible.api.generated.models.LocatorSchemaFlat
import app.indelible.api.generated.models.PatchHighlightBody
import app.indelible.api.generated.models.ReadingEventBody
import app.indelible.api.generated.models.ReadingPositionSchema
import app.indelible.api.generated.models.SourceLocatorSchemaFlat
import app.indelible.api.generated.models.UpsertNoteBody
import app.indelible.core.network.AuthenticatedApiTransport
import app.indelible.core.network.NOTE_PRECONDITION
import app.indelible.core.network.RawApiResponse
import app.indelible.core.network.ServerCapabilities
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
        session: Session,
        clientId: String,
        batch: List<OutboxRow>,
    ): SendOutcome
}

private const val REPLAYED_STATUS = 200
private const val NOT_FOUND_STATUS = 404
private const val CONFLICT_STATUS = 409
private const val NOTE_CONFLICT_CODE = "note_conflict"
private val SUCCESS_STATUS_RANGE = 200..299

class ApiOutboxSender(
    private val transport: AuthenticatedApiTransport,
    private val capabilities: ServerCapabilities,
) : OutboxSender {
    private val json = Json { ignoreUnknownKeys = true }

    override suspend fun send(
        session: Session,
        clientId: String,
        batch: List<OutboxRow>,
    ): SendOutcome {
        val result = runCatching { dispatch(session, clientId, batch) }
        return result.getOrElse { failure ->
            when (failure) {
                is CancellationException -> throw failure
                is StaleSessionException -> SendOutcome.Stale
                else -> SendOutcome.Transport(failure)
            }
        }
    }

    private suspend fun dispatch(
        session: Session,
        clientId: String,
        batch: List<OutboxRow>,
    ): SendOutcome {
        val head = batch.first()
        return when (head.kind) {
            OutboxKind.READING_EVENT -> sendReadingEvents(session, clientId, head.documentId, batch)
            OutboxKind.HIGHLIGHT_CREATE -> sendHighlightCreate(session, head)
            OutboxKind.HIGHLIGHT_COLOR -> sendHighlightColor(session, head)
            OutboxKind.HIGHLIGHT_NOTE -> sendHighlightNote(session, head)
            OutboxKind.HIGHLIGHT_TAGS -> sendHighlightTags(session, head)
            OutboxKind.HIGHLIGHT_DELETE -> sendHighlightDelete(session, head)
            OutboxKind.DOCUMENT_NOTE -> sendDocumentNote(session, head)
        }
    }

    private suspend fun sendReadingEvents(
        session: Session,
        clientId: String,
        documentId: String,
        batch: List<OutboxRow>,
    ): SendOutcome {
        val body =
            AppendReadingEventsBody(
                clientId = clientId,
                events = batch.map { readingEventBody(it.payload as OutboxPayload.ReadingEvent) },
            )
        val response =
            transport.rawAuthenticatedRequest(session) { client, baseUrl, token ->
                client.post("$baseUrl/api/v1/documents/$documentId/reading-events") {
                    authorize(token)
                    setBody(body)
                }
            }
        return outcome(response)
    }

    private suspend fun sendHighlightCreate(
        session: Session,
        row: OutboxRow,
    ): SendOutcome {
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
            transport.rawAuthenticatedRequest(session) { client, baseUrl, token ->
                client.post("$baseUrl/api/v1/documents/${row.documentId}/highlights") {
                    authorize(token)
                    setBody(body)
                }
            }
        return outcome(response, replayStatus = REPLAYED_STATUS)
    }

    private suspend fun sendHighlightColor(
        session: Session,
        row: OutboxRow,
    ): SendOutcome {
        val payload = row.payload as OutboxPayload.HighlightColor
        val response =
            transport.rawAuthenticatedRequest(session) { client, baseUrl, token ->
                client.patch("$baseUrl/api/v1/highlights/${payload.highlightId}") {
                    authorize(token)
                    setBody(PatchHighlightBody(color = payload.color))
                }
            }
        return outcome(response)
    }

    private suspend fun sendHighlightNote(
        session: Session,
        row: OutboxRow,
    ): SendOutcome {
        val payload = row.payload as OutboxPayload.HighlightNote
        val url = { baseUrl: String -> "$baseUrl/api/v1/highlights/${payload.highlightId}/note" }
        val noteBody = payload.body
        if (noteBody == null) {
            val response =
                transport.rawAuthenticatedRequest(session) { client, baseUrl, token ->
                    client.delete(url(baseUrl)) { authorize(token) }
                }
            return outcome(response, replayStatus = NOT_FOUND_STATUS)
        }
        val response =
            transport.rawAuthenticatedRequest(session) { client, baseUrl, token ->
                client.put(url(baseUrl)) {
                    authorize(token)
                    setBody(UpsertNoteBody(body = noteBody))
                }
            }
        return outcome(response)
    }

    private suspend fun sendHighlightTags(
        session: Session,
        row: OutboxRow,
    ): SendOutcome {
        val payload = row.payload as OutboxPayload.HighlightTags
        val response =
            transport.rawAuthenticatedRequest(session) { client, baseUrl, token ->
                client.put("$baseUrl/api/v1/highlights/${payload.highlightId}/tags") {
                    authorize(token)
                    setBody(HighlightTagsBody(tags = payload.tags))
                }
            }
        return outcome(response)
    }

    private suspend fun sendHighlightDelete(
        session: Session,
        row: OutboxRow,
    ): SendOutcome {
        val payload = row.payload as OutboxPayload.HighlightDelete
        val response =
            transport.rawAuthenticatedRequest(session) { client, baseUrl, token ->
                client.delete("$baseUrl/api/v1/highlights/${payload.highlightId}") { authorize(token) }
            }
        return outcome(response, replayStatus = NOT_FOUND_STATUS)
    }

    private suspend fun sendDocumentNote(
        session: Session,
        row: OutboxRow,
    ): SendOutcome {
        val payload = row.payload as OutboxPayload.DocumentNote
        val base = payload.baseUpdatedAtEpochMs
        if (base != null) preconditionMissing(session)?.let { return it }
        val response =
            transport.rawAuthenticatedRequest(session, readSuccessBody = true) { client, baseUrl, token ->
                client.put(baseUrl + "/api/v1/documents/${row.documentId}/note") {
                    authorize(token)
                    setBody(
                        DocumentUpsertNoteBody(
                            baseUpdatedAt = base?.let(Instant::fromEpochMilliseconds),
                            body = payload.body,
                            force = false,
                        ),
                    )
                }
            }
        return when {
            response.status in SUCCESS_STATUS_RANGE -> SendOutcome.Success(noteStamp(response))
            response.status == CONFLICT_STATUS -> noteConflict(response, payload.body) ?: outcome(response)
            else -> outcome(response)
        }
    }

    // An older server would take the base and overwrite anyway, so a checked note waits for one that lists it.
    private suspend fun preconditionMissing(session: Session): SendOutcome? =
        when (capabilities.supports(session, NOTE_PRECONDITION)) {
            true -> null
            false -> SendOutcome.Unsupported
            null -> SendOutcome.NoAnswer
        }

    /** The refused write's answer: ours already stored is a success, anything else is the conflict to resolve. */
    private fun noteConflict(
        response: RawApiResponse,
        body: String,
    ): SendOutcome? {
        val refused =
            runCatching { json.decodeFromString<DocumentNoteConflictResponse>(response.bodyText) }
                .getOrNull()
                ?.takeIf { it.code == NOTE_CONFLICT_CODE }
                ?: return null
        val serverAt = refused.note.updatedAt.toEpochMilliseconds()
        return when (refused.note.body) {
            body -> SendOutcome.Success(serverAt)
            else -> SendOutcome.NoteConflict(refused.note.body, serverAt)
        }
    }

    private fun noteStamp(response: RawApiResponse): Long? =
        runCatching { json.decodeFromString<DocumentNoteResponse>(response.bodyText) }
            .getOrNull()
            ?.updatedAt
            ?.toEpochMilliseconds()

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
            response.status in SUCCESS_STATUS_RANGE -> SendOutcome.Success()
            else ->
                SendOutcome.Http(
                    response.status,
                    response.retryAfterSeconds,
                    response.bodyText,
                    parseProblem(response.bodyText),
                )
        }
}

private fun HttpRequestBuilder.authorize(token: String) {
    header(HttpHeaders.Authorization, "Bearer $token")
}
