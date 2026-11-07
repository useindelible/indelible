package app.indelible.reader.repository

import app.indelible.api.generated.models.LocatorSchemaFlat
import app.indelible.core.offline.CachedHighlight
import app.indelible.core.offline.OfflineStore
import app.indelible.core.offline.OutboxKind
import app.indelible.core.offline.OutboxPayload
import app.indelible.core.offline.OutboxWorker
import app.indelible.core.offline.Session
import app.indelible.core.offline.applyToCachedHighlight
import app.indelible.core.util.highlightClientId
import app.indelible.core.util.readingEventId
import app.indelible.reader.model.HighlightData
import app.indelible.reader.model.HighlightLocator
import app.indelible.reader.model.HighlightNoteData
import app.indelible.reader.model.toHighlightLocator
import app.indelible.reader.model.toLocatorSchemaFlat
import kotlinx.datetime.Clock
import kotlinx.datetime.Instant
import kotlinx.serialization.json.Json
import kotlin.math.roundToInt

private const val BASIS_POINTS_PER_PERCENT = 100
private const val MAX_BASIS_POINTS = 10_000
private const val EVENT_CAUSE = "reader"

private data class HighlightTarget(
    val session: Session,
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
    suspend fun createHighlight(
        session: Session,
        documentId: String,
        color: String,
        textContent: String,
        locator: HighlightLocator,
    ): HighlightData {
        val highlightId = highlightClientId()
        val at = now()
        val payload =
            OutboxPayload.HighlightCreate(
                highlightId = highlightId,
                color = color,
                textContent = textContent,
                locatorJson = locatorJson.encodeToString(LocatorSchemaFlat.serializer(), locator.toLocatorSchemaFlat()),
                sourceLocatorJson = null,
            )
        val cached =
            store.enqueue(session, OutboxKind.HIGHLIGHT_CREATE, highlightId, documentId) {
                payload to applyToCachedHighlight(highlightId, documentId, payload, at)
            }
        drain()
        return checkNotNull(cached).toHighlightData(at)
    }

    suspend fun updateHighlightColor(
        session: Session,
        documentId: String,
        highlightId: String,
        color: String,
    ): HighlightData {
        val at = now()
        val patched =
            patchCachedHighlight(
                target = HighlightTarget(session, documentId, highlightId),
                kind = OutboxKind.HIGHLIGHT_COLOR,
                payload = OutboxPayload.HighlightColor(highlightId, color),
            )
        return patched?.toHighlightData(at)
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
        session: Session,
        documentId: String,
        highlightId: String,
        body: String,
    ): HighlightNoteData {
        val at = now()
        patchCachedHighlight(
            target = HighlightTarget(session, documentId, highlightId),
            kind = OutboxKind.HIGHLIGHT_NOTE,
            payload = OutboxPayload.HighlightNote(highlightId, body),
        )
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
        session: Session,
        documentId: String,
        highlightId: String,
    ) {
        patchCachedHighlight(
            target = HighlightTarget(session, documentId, highlightId),
            kind = OutboxKind.HIGHLIGHT_NOTE,
            payload = OutboxPayload.HighlightNote(highlightId, null),
        )
    }

    suspend fun setHighlightTags(
        session: Session,
        documentId: String,
        highlightId: String,
        tags: List<String>,
    ) {
        patchCachedHighlight(
            target = HighlightTarget(session, documentId, highlightId),
            kind = OutboxKind.HIGHLIGHT_TAGS,
            payload = OutboxPayload.HighlightTags(highlightId, tags),
        )
    }

    suspend fun deleteHighlight(
        session: Session,
        documentId: String,
        highlightId: String,
    ) {
        patchCachedHighlight(
            target = HighlightTarget(session, documentId, highlightId),
            kind = OutboxKind.HIGHLIGHT_DELETE,
            payload = OutboxPayload.HighlightDelete(highlightId),
        )
    }

    suspend fun upsertDocumentNote(
        session: Session,
        documentId: String,
        body: String,
    ) {
        store.enqueue(session, OutboxKind.DOCUMENT_NOTE, documentId, documentId) {
            setCachedNote(documentId, body)
            OutboxPayload.DocumentNote(body = body, baseUpdatedAtEpochMs = null) to Unit
        }
        drain()
    }

    suspend fun readingEvent(
        session: Session,
        documentId: String,
        kind: String,
        sessionId: String?,
        progressBasisPoints: Int?,
    ) {
        val recordedAt = now()
        val eventId = readingEventId()
        store.enqueue(session, OutboxKind.READING_EVENT, documentId, documentId) {
            progressBasisPoints?.let { patchCachedProgress(documentId, it / BASIS_POINTS_PER_PERCENT) }
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
    ): CachedHighlight? {
        val (session, documentId, highlightId) = target
        val at = now()
        val patched =
            store.enqueue(session, kind, highlightId, documentId) {
                payload to applyToCachedHighlight(highlightId, documentId, payload, at)
            }
        drain()
        return patched
    }

    private fun drain() {
        worker.requestDrain()
    }
}

/** The reader's highlight from its cached shape; a time the cache does not hold reads as [at]. */
internal fun CachedHighlight.toHighlightData(at: Long): HighlightData {
    val created = Instant.fromEpochMilliseconds(createdAtEpochMs ?: at)
    val updated = Instant.fromEpochMilliseconds(updatedAtEpochMs ?: at)
    return HighlightData(
        id = id,
        color = color,
        textContent = textContent,
        locator =
            locator?.let {
                locatorJson.decodeFromString(LocatorSchemaFlat.serializer(), it).toHighlightLocator()
            },
        tags = tags,
        createdAt = created,
        updatedAt = updated,
        documentId = documentId,
        note =
            note?.let {
                // The server mints the durable note id when the row syncs; the highlight id stands in.
                HighlightNoteData(id = id, highlightId = id, body = it.body, createdAt = created, updatedAt = updated)
            },
    )
}

private val locatorJson = Json { ignoreUnknownKeys = true }

internal fun basisPoints(percent: Float): Int {
    val points = (percent * BASIS_POINTS_PER_PERCENT).roundToInt()
    return points.coerceIn(0, MAX_BASIS_POINTS)
}
