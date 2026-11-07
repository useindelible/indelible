package app.indelible.core.offline

import app.cash.sqldelight.coroutines.asFlow
import app.cash.sqldelight.coroutines.mapToList
import app.indelible.db.List_snapshot
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.jsonObject

internal class SqlDelightListSnapshots(
    private val context: SqlDelightStoreContext,
) : ListSnapshotStore {
    override suspend fun replaceList(
        session: Session,
        key: String,
        rows: List<SnapshotRow>,
        at: Long,
    ) {
        context.write {
            context.database.transaction {
                context.requireLive(session)
                context.queries.deleteSnapshotList(session.scope, key)
                insert(session.scope, key, rows, from = 0, at = at)
            }
        }
    }

    override suspend fun appendList(
        session: Session,
        key: String,
        rows: List<SnapshotRow>,
        at: Long,
    ) {
        context.write {
            context.database.transaction {
                context.requireLive(session)
                val from = context.queries.nextSnapshotPosition(session.scope, key).executeAsOne()
                insert(session.scope, key, rows, from, at)
            }
        }
    }

    override suspend fun list(
        scope: String,
        key: String,
    ): List<SnapshotRow> =
        context.read {
            context.queries
                .snapshotRows(scope, key)
                .executeAsList()
                .map(::snapshotRowFrom)
        }

    override fun observeList(
        scope: String,
        key: String,
    ): Flow<KeptList> =
        context.queries
            .snapshotRows(scope, key)
            .asFlow()
            .mapToList(context.dispatcher)
            .map { rows -> KeptList(rows.map(::snapshotRowFrom), rows.maxOfOrNull { it.seen_at }) }

    override suspend fun lastSeen(
        scope: String,
        documentId: String,
    ): SnapshotRow? =
        context.read {
            context.queries
                .lastSeenRow(scope, documentId)
                .executeAsOneOrNull()
                ?.let(::snapshotRowFrom)
        }

    override suspend fun forget(
        session: Session,
        id: String,
        keep: (String) -> Boolean,
    ) {
        context.write {
            context.database.transaction {
                context.requireLive(session)
                val keys =
                    context.queries
                        .snapshotKeys(session.scope)
                        .executeAsList()
                        .filterNot(keep)
                if (keys.isNotEmpty()) context.queries.forgetSnapshotRows(session.scope, keys, id)
            }
        }
    }

    override suspend fun setTriageState(
        session: Session,
        documentId: String,
        state: String,
    ) {
        context.write {
            context.database.transaction {
                context.requireLive(session)
                val queries = context.queries
                queries.libraryRowsOfDocument(session.scope, documentId).executeAsList().forEach { row ->
                    row.row_json.withTriageState(state)?.let {
                        queries.setSnapshotRowJson(it, row.scope, row.list_key, row.position)
                    }
                }
                queries
                    .cachedReaderJson(session.scope, documentId)
                    .executeAsOneOrNull()
                    ?.withTriageState(state)
                    ?.let { queries.setCachedReaderJson(it, session.scope, documentId) }
            }
        }
    }

    private fun insert(
        scope: String,
        key: String,
        rows: List<SnapshotRow>,
        from: Long,
        at: Long,
    ) {
        rows.forEachIndexed { index, row ->
            context.queries.insertSnapshotRow(
                scope = scope,
                list_key = key,
                position = from + index,
                item_id = row.itemId,
                document_id = row.documentId,
                document_type = row.documentType,
                title = row.title,
                row_json = row.rowJson,
                seen_at = at,
            )
        }
    }
}

private fun String.withTriageState(state: String): String? {
    val fields = runCatching { offlineJson.parseToJsonElement(this).jsonObject }.getOrNull() ?: return null
    return JsonObject(fields + ("triage_state" to JsonPrimitive(state))).toString()
}

private fun snapshotRowFrom(row: List_snapshot): SnapshotRow =
    SnapshotRow(
        itemId = row.item_id,
        documentId = row.document_id,
        documentType = row.document_type,
        title = row.title,
        rowJson = row.row_json,
    )
