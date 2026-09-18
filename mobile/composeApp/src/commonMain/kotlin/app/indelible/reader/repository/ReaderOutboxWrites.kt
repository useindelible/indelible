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
    private val json = Json { ignoreUnknownKeys = true }

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
                locatorJson = json.encodeToString(LocatorSchemaFlat.serializer(), locator.toLocatorSchemaFlat()),
                sourceLocatorJson = null,
            )
        val cached =
            store.enqueue(session, OutboxKind.HIGHLIGHT_CREATE, highlightId, documentId) {
                payload to applyToCachedHighlight(highlightId, documentId, payload, at)
            }
        drain()
        return highlightFrom(checkNotNull(cached), at)
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
}

internal fun basisPoints(percent: Float): Int {
    val points = (percent * BASIS_POINTS_PER_PERCENT).roundToInt()
    return points.coerceIn(0, MAX_BASIS_POINTS)
}
