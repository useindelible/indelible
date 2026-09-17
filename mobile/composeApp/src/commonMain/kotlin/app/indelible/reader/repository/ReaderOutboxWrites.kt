package app.indelible.reader.repository

import app.indelible.api.generated.models.LocatorSchemaFlat
import app.indelible.core.offline.OfflineStore
import app.indelible.core.offline.OutboxKind
import app.indelible.core.offline.OutboxPayload
import app.indelible.core.offline.OutboxWorker
import app.indelible.core.util.highlightClientId
import app.indelible.core.util.readingEventId
import app.indelible.reader.model.HighlightData
import app.indelible.reader.model.HighlightLocator
import app.indelible.reader.model.HighlightNoteData
import app.indelible.reader.model.toHighlightLocator
import app.indelible.reader.model.toLocatorSchemaFlat
import kotlinx.datetime.Clock
import kotlinx.datetime.Instant
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import kotlin.math.roundToInt

private const val BASIS_POINTS_PER_PERCENT = 100
private const val MAX_BASIS_POINTS = 10_000
private const val EVENT_CAUSE = "reader"

/** The locally cached shape of a highlight; mutations patch its fields inside their transaction. */
@Serializable
internal data class CachedHighlight(
    val id: String,
    val documentId: String,
    val color: String,
    val textContent: String,
    val locator: String? = null,
    val tags: List<String> = emptyList(),
    val note: CachedHighlightNote? = null,
)

@Serializable
internal data class CachedHighlightNote(
    val body: String,
)

private data class HighlightTarget(
    val scope: String,
    val documentId: String,
    val highlightId: String,
)

/**
 * The local half of every reader write: the cache change and the outbox row are one transaction,
 * and the drain request that follows never blocks, so a write returns at local-write speed.
 */
internal class ReaderOutboxWrites(
    private val store: OfflineStore,
    private val worker: OutboxWorker,
    private val now: () -> Long = { Clock.System.now().toEpochMilliseconds() },
) {
    private val json = Json { ignoreUnknownKeys = true }

    suspend fun createHighlight(
        scope: String,
        documentId: String,
        color: String,
        textContent: String,
        locator: HighlightLocator,
    ): HighlightData {
        val highlightId = highlightClientId()
        val at = now()
        val locatorJson = json.encodeToString(LocatorSchemaFlat.serializer(), locator.toLocatorSchemaFlat())
        val cached =
            CachedHighlight(
                id = highlightId,
                documentId = documentId,
                color = color,
                textContent = textContent,
                locator = locatorJson,
            )
        store.enqueue(scope, OutboxKind.HIGHLIGHT_CREATE, highlightId, documentId) {
            upsertCachedHighlight(
                highlightId,
                documentId,
                json.encodeToString(CachedHighlight.serializer(), cached),
                at,
            )
            OutboxPayload.HighlightCreate(
                highlightId = highlightId,
                color = color,
                textContent = textContent,
                locatorJson = locatorJson,
                sourceLocatorJson = null,
            ) to Unit
        }
        drain()
        return highlightFrom(cached, at)
    }

    suspend fun updateHighlightColor(
        scope: String,
        documentId: String,
        highlightId: String,
        color: String,
    ): HighlightData {
        val at = now()
        val patched =
            patchCachedHighlight(
                target = HighlightTarget(scope, documentId, highlightId),
                kind = OutboxKind.HIGHLIGHT_COLOR,
                payload = OutboxPayload.HighlightColor(highlightId, color),
            ) { it.withField("color", JsonPrimitive(color)) }
        return patched?.let { highlightFrom(it, at) }
            ?: HighlightData(
                id = highlightId,
                color = color,
                textContent = "",
                locator = null,
                tags = emptyList(),
                createdAt = Instant.fromEpochMilliseconds(at),
                updatedAt = Instant.fromEpochMilliseconds(at),
            )
    }

    suspend fun upsertHighlightNote(
        scope: String,
        documentId: String,
        highlightId: String,
        body: String,
    ): HighlightNoteData {
        val at = now()
        patchCachedHighlight(
            target = HighlightTarget(scope, documentId, highlightId),
            kind = OutboxKind.HIGHLIGHT_NOTE,
            payload = OutboxPayload.HighlightNote(highlightId, body),
        ) { it.withField("note", JsonObject(mapOf("body" to JsonPrimitive(body)))) }
        return HighlightNoteData(
            // The server mints the durable note id when the row syncs; this one only backs local state.
            id = highlightId,
            highlightId = highlightId,
            body = body,
            createdAt = Instant.fromEpochMilliseconds(at),
            updatedAt = Instant.fromEpochMilliseconds(at),
        )
    }

    suspend fun deleteHighlightNote(
        scope: String,
        documentId: String,
        highlightId: String,
    ) {
        patchCachedHighlight(
            target = HighlightTarget(scope, documentId, highlightId),
            kind = OutboxKind.HIGHLIGHT_NOTE,
            payload = OutboxPayload.HighlightNote(highlightId, null),
        ) { it.withField("note", null) }
    }

    suspend fun setHighlightTags(
        scope: String,
        documentId: String,
        highlightId: String,
        tags: List<String>,
    ) {
        patchCachedHighlight(
            target = HighlightTarget(scope, documentId, highlightId),
            kind = OutboxKind.HIGHLIGHT_TAGS,
            payload = OutboxPayload.HighlightTags(highlightId, tags),
        ) { it.withField("tags", JsonArray(tags.map(::JsonPrimitive))) }
    }

    suspend fun deleteHighlight(
        scope: String,
        documentId: String,
        highlightId: String,
    ) {
        store.enqueue(scope, OutboxKind.HIGHLIGHT_DELETE, highlightId, documentId) {
            deleteCachedHighlight(highlightId)
            OutboxPayload.HighlightDelete(highlightId) to Unit
        }
        drain()
    }

    suspend fun upsertDocumentNote(
        scope: String,
        documentId: String,
        body: String,
    ) {
        store.enqueue(scope, OutboxKind.DOCUMENT_NOTE, documentId, documentId) {
            OutboxPayload.DocumentNote(body = body, baseUpdatedAtEpochMs = null) to Unit
        }
        drain()
    }

    suspend fun readingEvent(
        scope: String,
        documentId: String,
        kind: String,
        sessionId: String?,
        progressBasisPoints: Int?,
    ) {
        val recordedAt = now()
        val eventId = readingEventId()
        store.enqueue(scope, OutboxKind.READING_EVENT, documentId, documentId) {
            OutboxPayload.ReadingEvent(
                eventId = eventId,
                originSeq = allocateOriginSeq(),
                kind = kind,
                progressBasisPoints = progressBasisPoints,
                cause = EVENT_CAUSE,
                sessionId = sessionId,
                attempt = null,
                positionJson = null,
                assetKind = null,
                activeMs = null,
                recordedAtEpochMs = recordedAt,
            ) to Unit
        }
        drain()
    }

    private suspend fun patchCachedHighlight(
        target: HighlightTarget,
        kind: OutboxKind,
        payload: OutboxPayload,
        patch: (JsonObject) -> JsonObject,
    ): CachedHighlight? {
        val (scope, documentId, highlightId) = target
        val at = now()
        val patched =
            store.enqueue(scope, kind, highlightId, documentId) {
                val cached = getCachedHighlight(highlightId)
                val updated = cached?.let { patch(json.parseToJsonElement(it.payloadJson) as JsonObject) }
                if (cached != null && updated != null) {
                    upsertCachedHighlight(highlightId, documentId, updated.toString(), at)
                }
                payload to updated
            }
        drain()
        return patched?.let { json.decodeFromJsonElement(CachedHighlight.serializer(), it) }
    }

    private fun drain() {
        worker.requestDrain()
    }

    private fun highlightFrom(
        cached: CachedHighlight,
        at: Long,
    ): HighlightData =
        HighlightData(
            id = cached.id,
            color = cached.color,
            textContent = cached.textContent,
            locator =
                cached.locator?.let {
                    json.decodeFromString(LocatorSchemaFlat.serializer(), it).toHighlightLocator()
                },
            tags = cached.tags,
            createdAt = Instant.fromEpochMilliseconds(at),
            updatedAt = Instant.fromEpochMilliseconds(at),
            documentId = cached.documentId,
            note =
                cached.note?.let {
                    HighlightNoteData(
                        id = cached.id,
                        highlightId = cached.id,
                        body = it.body,
                        createdAt = Instant.fromEpochMilliseconds(at),
                        updatedAt = Instant.fromEpochMilliseconds(at),
                    )
                },
        )

    private fun JsonObject.withField(
        name: String,
        value: JsonElement?,
    ): JsonObject = JsonObject(if (value == null) this - name else this + (name to value))
}

internal fun basisPoints(percent: Float): Int {
    val points = (percent * BASIS_POINTS_PER_PERCENT).roundToInt()
    return points.coerceIn(0, MAX_BASIS_POINTS)
}
