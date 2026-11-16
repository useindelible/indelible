package app.indelible.core.offline

import app.indelible.db.OfflineQueries
import kotlinx.serialization.encodeToString

private val NO_REVISION = DocumentRevision(content = 0, position = 0, ackedProgressSeq = -1)

internal fun OfflineQueries.revision(
    scope: String,
    documentId: String,
): DocumentRevision =
    documentRevision(scope, documentId)
        .executeAsOneOrNull()
        ?.let { DocumentRevision(it.content_rev, it.position_rev, it.acked_progress_seq) }
        ?: NO_REVISION

internal fun OfflineQueries.localChanges(
    scope: String,
    documentId: String,
): LocalChanges =
    LocalChanges(
        revision = revision(scope, documentId),
        rows = liveOutboxForDocument(scope, documentId).executeAsList().map(::outboxRowFrom),
    )

internal fun OfflineQueries.staleness(
    scope: String,
    documentId: String,
    fetchedAt: DocumentRevision,
): Staleness {
    val now = revision(scope, documentId)
    return Staleness(content = now.content != fetchedAt.content, position = now.position != fetchedAt.position)
}

/** Moves the revision the change affects; reading events without progress change nothing. */
internal fun OfflineQueries.moveRevision(
    scope: String,
    documentId: String,
    payload: OutboxPayload,
) {
    val event = payload as? OutboxPayload.ReadingEvent
    if (event != null && event.progressBasisPoints == null) return
    ensureRevision(scope, documentId)
    if (event != null) bumpPositionRevision(scope, documentId) else bumpContentRevision(scope, documentId)
}

// An older same-field row here failed or was retried after, so it is superseded, never replayed over the server value.
internal fun OfflineQueries.acknowledge(
    row: OutboxRow,
    ackedNoteUpdatedAt: Long? = null,
) {
    deleteOutboxRow(row.scope, row.id)
    if (row.kind == OutboxKind.DOCUMENT_NOTE && ackedNoteUpdatedAt != null) {
        rebaseNotes(row, ackedNoteUpdatedAt)
    }
    if (row.kind.isHighlight()) acknowledgeHighlight(row)
    when (row.kind) {
        OutboxKind.DOCUMENT_NOTE,
        OutboxKind.HIGHLIGHT_COLOR,
        OutboxKind.HIGHLIGHT_NOTE,
        OutboxKind.HIGHLIGHT_TAGS,
        -> supersedeField(row.scope, row.entityId, row.kind.wireName(), row.seq)
        OutboxKind.HIGHLIGHT_DELETE -> supersedeHighlight(row.scope, row.entityId, row.seq)
        OutboxKind.HIGHLIGHT_CREATE, OutboxKind.READING_EVENT -> Unit
    }
    moveRevision(row.scope, row.documentId, row.payload)
    val event = row.payload as? OutboxPayload.ReadingEvent
    if (event?.progressBasisPoints != null) {
        raiseAckedProgressSeq(seq = event.originSeq, scope = row.scope, document_id = row.documentId)
    }
}

private fun OfflineQueries.acknowledgeHighlight(acked: OutboxRow) {
    val server = serverHighlight(acked.scope, acked.entityId).executeAsOneOrNull()?.let(::decodeHighlight)
    when (val next = applyPayload(server, acked.payload, acked.documentId)) {
        null -> deleteServerHighlight(acked.scope, acked.entityId)
        else -> upsertServerHighlight(acked.scope, acked.entityId, acked.documentId, encodeHighlight(next))
    }
}

/** Puts the server's highlights back in the copy with the live queued changes replayed on top. */
internal fun OfflineQueries.rebuildCachedHighlights(
    scope: String,
    documentId: String,
) {
    val server = serverHighlightsForDocument(scope, documentId).executeAsList().map(::decodeHighlight)
    writeCachedHighlights(scope, documentId, localChanges(scope, documentId).highlights(server))
}

internal fun OfflineQueries.writeCachedHighlights(
    scope: String,
    documentId: String,
    highlights: List<CachedHighlight>,
) {
    deleteHighlightsForDocument(scope, documentId)
    highlights.forEach { highlight ->
        val updatedAt = highlight.updatedAtEpochMs ?: 0L
        upsertCachedHighlight(scope, highlight.id, documentId, encodeHighlight(highlight), updatedAt)
    }
}

private val highlightSerializer = CachedHighlight.serializer()

internal fun encodeHighlight(value: CachedHighlight): String = offlineJson.encodeToString(highlightSerializer, value)

private fun decodeHighlight(json: String): CachedHighlight = offlineJson.decodeFromString(highlightSerializer, json)

// The acked text is now the server's; a later note that had conflicted was judged against a version that
// is gone, so it goes back to pending for the sender to check again.
private fun OfflineQueries.rebaseNotes(
    acked: OutboxRow,
    serverUpdatedAt: Long,
) {
    val body = (acked.payload as OutboxPayload.DocumentNote).body
    setCachedServerNoteStamp(body, serverUpdatedAt, acked.scope, acked.documentId)
    for (later in laterLiveNotesForDocument(acked.scope, acked.documentId, acked.seq).executeAsList()) {
        val row = outboxRowFrom(later)
        val rebased = (row.payload as OutboxPayload.DocumentNote).copy(baseUpdatedAtEpochMs = serverUpdatedAt)
        setPayloadJson(offlineJson.encodeToString<OutboxPayload>(rebased), row.scope, row.id)
        if (row.failure?.kind == SyncFailure.Kind.NOTE_CONFLICT) resetForRetry(row.scope, row.id)
    }
}
