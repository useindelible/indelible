package app.indelible.core.offline

import app.indelible.db.OfflineQueries

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

/**
 * Moves the revision a change to the document's live rows affects. Reading events without
 * progress never change what the reader shows, so they move nothing.
 */
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

/**
 * The acknowledgement transaction's body. Per-entity send order means an older row that sets the
 * same field can only still be here because it failed, or was retried after this one was sent, so
 * it is superseded rather than ever replayed or sent over the value the server now holds.
 */
internal fun OfflineQueries.acknowledge(row: OutboxRow) {
    deleteOutboxRow(row.scope, row.id)
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
