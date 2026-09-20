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

/** Moves on every enqueue and acknowledgement, so a response fetched across a move may be stale. */
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

    /** The newest unacknowledged queued position and the highest one reached, else the server's. */
    fun progress(server: Progress): Progress {
        val queued =
            live()
                .mapNotNull { it.payload as? OutboxPayload.ReadingEvent }
                .filter { it.originSeq > revision.ackedProgressSeq }
                .mapNotNull { it.progressBasisPoints }
        val latest = queued.lastOrNull() ?: return server
        val percent = latest / BASIS_POINTS_PER_PERCENT
        val reached = queued.max() / BASIS_POINTS_PER_PERCENT
        return Progress(percent = percent, maxPercent = maxOf(server.maxPercent ?: percent, reached))
    }

    private fun live(): List<OutboxRow> = rows.filterNot { it.superseded }
}

/** Null when the highlight does not exist; a create keeps an existing one, which may carry later edits. */
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

/** Applies [payload] inside the enqueue transaction, so the cache always matches the replayed view. */
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
