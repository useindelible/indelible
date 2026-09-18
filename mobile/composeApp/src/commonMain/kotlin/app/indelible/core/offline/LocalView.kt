package app.indelible.core.offline

import kotlinx.serialization.Serializable

/** The locally held shape of a highlight: the server's, with this device's queued changes applied. */
@Serializable
data class CachedHighlight(
    val id: String,
    val documentId: String,
    val color: String,
    val textContent: String,
    val locator: String? = null,
    val tags: List<String> = emptyList(),
    val note: CachedHighlightNote? = null,
    val createdAtEpochMs: Long? = null,
    val updatedAtEpochMs: Long? = null,
)

@Serializable
data class CachedHighlightNote(
    val body: String,
)

data class Progress(
    val percent: Int?,
    val maxPercent: Int?,
)

/**
 * Moves whenever a document's live outbox rows change (on enqueue and on acknowledgement), so a
 * server response fetched before the move may predate a change the live rows no longer show.
 */
data class DocumentRevision(
    val content: Long,
    val position: Long,
    val ackedProgressSeq: Long,
)

/** A document's live, unsuperseded outbox rows in seq order, read together with their revision. */
data class LocalChanges(
    val revision: DocumentRevision,
    val rows: List<OutboxRow>,
) {
    /** The server list with every live highlight change replayed on top, in seq order. */
    fun highlights(server: List<CachedHighlight>): List<CachedHighlight> {
        val byId = LinkedHashMap<String, CachedHighlight>()
        server.forEach { byId[it.id] = it }
        for (row in live()) {
            if (!row.kind.isHighlight()) continue
            val next = applyPayload(byId[row.entityId], row.payload, row.documentId)
            if (next == null) byId.remove(row.entityId) else byId[row.entityId] = next
        }
        return byId.values.toList()
    }

    fun note(server: String?): String? =
        live()
            .lastOrNull { it.kind == OutboxKind.DOCUMENT_NOTE }
            ?.let { (it.payload as OutboxPayload.DocumentNote).body }
            ?: server

    /**
     * The newest queued progress this device has not yet had acknowledged, else the server's.
     * An acknowledged event is already in the server's projection, and an older one never wins it.
     */
    fun progress(server: Progress): Progress {
        val basisPoints =
            live()
                .mapNotNull { it.payload as? OutboxPayload.ReadingEvent }
                .lastOrNull { it.progressBasisPoints != null && it.originSeq > revision.ackedProgressSeq }
                ?.progressBasisPoints
                ?: return server
        val percent = basisPoints / BASIS_POINTS_PER_PERCENT
        return Progress(percent = percent, maxPercent = maxOf(server.maxPercent ?: percent, percent))
    }

    private fun live(): List<OutboxRow> = rows.filterNot { it.superseded }
}

/**
 * One highlight change applied to its cached value; null means the highlight does not exist.
 * A create over an existing highlight keeps it, since the server already has the create and
 * possibly later changes to it; an edit of an unknown highlight has nothing to change.
 */
fun applyPayload(
    cached: CachedHighlight?,
    payload: OutboxPayload,
    documentId: String,
): CachedHighlight? =
    when (payload) {
        is OutboxPayload.HighlightCreate ->
            cached ?: CachedHighlight(
                id = payload.highlightId,
                documentId = documentId,
                color = payload.color,
                textContent = payload.textContent,
                locator = payload.locatorJson,
            )
        is OutboxPayload.HighlightColor -> cached?.copy(color = payload.color)
        is OutboxPayload.HighlightNote -> cached?.copy(note = payload.body?.let(::CachedHighlightNote))
        is OutboxPayload.HighlightTags -> cached?.copy(tags = payload.tags)
        is OutboxPayload.HighlightDelete -> null
        is OutboxPayload.ReadingEvent, is OutboxPayload.DocumentNote -> cached
    }

/**
 * Applies [payload] to the cached highlight it targets, inside the enqueue transaction, and returns
 * the result, so the cache always matches what replaying the queue over the server would show.
 */
fun EnqueueTx.applyToCachedHighlight(
    highlightId: String,
    documentId: String,
    payload: OutboxPayload,
    at: Long,
): CachedHighlight? {
    val serializer = CachedHighlight.serializer()
    val cached = getCachedHighlight(highlightId)?.let { offlineJson.decodeFromString(serializer, it.payloadJson) }
    val next = applyPayload(cached, payload, documentId)
    when {
        next != null -> upsertCachedHighlight(highlightId, documentId, offlineJson.encodeToString(serializer, next), at)
        cached != null -> deleteCachedHighlight(highlightId)
    }
    return next
}

internal fun OutboxKind.isHighlight(): Boolean =
    when (this) {
        OutboxKind.HIGHLIGHT_CREATE,
        OutboxKind.HIGHLIGHT_COLOR,
        OutboxKind.HIGHLIGHT_NOTE,
        OutboxKind.HIGHLIGHT_TAGS,
        OutboxKind.HIGHLIGHT_DELETE,
        -> true
        OutboxKind.READING_EVENT, OutboxKind.DOCUMENT_NOTE -> false
    }

private const val BASIS_POINTS_PER_PERCENT = 100
