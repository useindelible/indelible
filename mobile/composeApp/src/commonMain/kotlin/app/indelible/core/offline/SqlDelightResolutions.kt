package app.indelible.core.offline

import kotlinx.serialization.encodeToString

internal class SqlDelightResolutions(
    private val context: SqlDelightStoreContext,
    private val events: LocalChangeEvents,
) : OutboxResolutionStore {
    private val queries get() = context.queries

    override suspend fun discardRows(
        session: Session,
        rowIds: List<String>,
    ): Int {
        val documents = mutableSetOf<String>()
        val deleted =
            context.write {
                context.database.transactionWithResult {
                    context.requireLive(session)
                    rowIds.sumOf { discard(session.scope, it, documents) }
                }
            }
        documents.forEach(events::emit)
        return deleted
    }

    override suspend fun keepLocalNote(
        session: Session,
        rowId: String,
        seenServerUpdatedAt: Long,
    ): Boolean =
        resolveConflict(session, rowId, seenServerUpdatedAt) { row ->
            val rebased = (row.payload as OutboxPayload.DocumentNote).copy(baseUpdatedAtEpochMs = seenServerUpdatedAt)
            queries.setPayloadJson(offlineJson.encodeToString<OutboxPayload>(rebased), row.scope, row.id)
            queries.resetForRetry(row.scope, row.id)
        }

    override suspend fun keepServerNote(
        session: Session,
        rowId: String,
        seenServerUpdatedAt: Long,
    ): Boolean =
        resolveConflict(session, rowId, seenServerUpdatedAt) { row ->
            queries.deleteOutboxRow(row.scope, row.id)
            val theirs = row.failure?.serverNoteBody
            queries.setCachedServerNoteStamp(theirs, seenServerUpdatedAt, row.scope, row.documentId)
            rebuildNote(row.scope, row.documentId)
            queries.moveRevision(row.scope, row.documentId, row.payload)
        }

    /** Applies [resolve] only while the row is still the conflict the sheet showed, and announces the change. */
    private suspend fun resolveConflict(
        session: Session,
        rowId: String,
        seenServerUpdatedAt: Long,
        resolve: (OutboxRow) -> Unit,
    ): Boolean {
        val resolved =
            context.write {
                context.database.transactionWithResult {
                    context.requireLive(session)
                    val row = queries.outboxRow(session.scope, rowId).executeAsOneOrNull()?.let(::outboxRowFrom)
                    row?.takeIf { it.isConflictAt(seenServerUpdatedAt) }?.also(resolve)
                }
            }
        resolved?.let { events.emit(it.documentId) }
        return resolved != null
    }

    private fun discard(
        scope: String,
        id: String,
        documents: MutableSet<String>,
    ): Int {
        val row = queries.outboxRow(scope, id).executeAsOneOrNull()?.let(::outboxRowFrom) ?: return 0
        queries.deleteOutboxRow(scope, id)
        var deleted = 1
        when (row.kind) {
            OutboxKind.HIGHLIGHT_CREATE -> {
                queries.deleteBlockedRowsForEntity(scope, row.entityId)
                deleted += queries.changedRows().executeAsOne().toInt()
                queries.deleteCachedHighlight(scope, row.entityId)
            }
            OutboxKind.DOCUMENT_NOTE -> rebuildNote(scope, row.documentId)
            else -> Unit
        }
        queries.moveRevision(scope, row.documentId, row.payload)
        documents += row.documentId
        return deleted
    }

    // The copy shows the newest live queued note, else the server's; a row that leaves takes its text with it.
    private fun rebuildNote(
        scope: String,
        documentId: String,
    ) {
        val serverBody = queries.cachedDocument(scope, documentId).executeAsOneOrNull()?.note_server_body
        queries.setCachedNote(queries.localChanges(scope, documentId).note(serverBody), scope, documentId)
    }
}

private fun OutboxRow.isConflictAt(seenServerUpdatedAt: Long): Boolean {
    val shown = failure?.takeIf { it.kind == SyncFailure.Kind.NOTE_CONFLICT } ?: return false
    return state == OutboxState.FAILED && !superseded && shown.serverNoteUpdatedAtEpochMs == seenServerUpdatedAt
}
