package app.indelible.core.offline

import app.cash.sqldelight.coroutines.asFlow
import app.cash.sqldelight.coroutines.mapToList
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map

internal class SqlDelightCachedDocuments(
    private val context: SqlDelightStoreContext,
) : CachedDocumentStore {
    override suspend fun upsertCachedDocument(
        scope: String,
        row: CachedDocumentRow,
    ) {
        context.write {
            context.queries.upsertCachedDocument(
                scope = scope,
                document_id = row.documentId,
                document_type = row.documentType,
                title = row.title,
                reader_json = row.readerJson,
                pinned = row.pinned.toLong(),
                last_opened_at = row.lastOpenedAt,
                last_synced_at = row.lastSyncedAt,
                bytes = row.bytes,
            )
        }
    }

    override suspend fun touchDocumentOpened(
        scope: String,
        documentId: String,
        at: Long,
    ) {
        context.write {
            context.queries.touchDocumentOpened(at, scope, documentId)
        }
    }

    override suspend fun setPinned(
        scope: String,
        documentId: String,
        pinned: Boolean,
    ): Boolean =
        context.write {
            context.database.transactionWithResult {
                context.queries.setPinned(pinned.toLong(), scope, documentId)
                context.queries.changedRows().executeAsOne() > 0
            }
        }

    override suspend fun setDocumentBytes(
        scope: String,
        documentId: String,
        bytes: Long,
    ) {
        context.write {
            context.queries.setDocumentBytes(bytes, scope, documentId)
        }
    }

    override suspend fun cachedDocuments(scope: String): List<CachedDocumentRow> =
        context.read {
            context.queries
                .cachedDocuments(scope)
                .executeAsList()
                .map(::cachedDocumentRowFrom)
        }

    override suspend fun cachedDocument(
        scope: String,
        documentId: String,
    ): CachedDocumentRow? =
        context.read {
            context.queries
                .cachedDocument(scope, documentId)
                .executeAsOneOrNull()
                ?.let(::cachedDocumentRowFrom)
        }

    override suspend fun unpinnedLru(scope: String): List<CachedDocumentRow> =
        context.read {
            context.queries
                .unpinnedLru(scope)
                .executeAsList()
                .map(::cachedDocumentRowFrom)
        }

    override suspend fun totalBytes(scope: String): Long =
        context.read {
            context.queries
                .totalBytes(scope)
                .executeAsOne()
                .SUM ?: 0L
        }

    override fun observeCatalog(scope: String): Flow<List<CatalogEntry>> =
        context.queries
            .catalog(scope)
            .asFlow()
            .mapToList(context.dispatcher)
            .map { rows ->
                rows.map { row ->
                    CatalogEntry(
                        documentId = row.document_id,
                        documentType = row.document_type,
                        title = row.title,
                        pinned = row.pinned != 0L,
                        lastOpenedAt = row.last_opened_at,
                        lastSyncedAt = row.last_synced_at,
                        bytes = row.bytes,
                        generation = row.generation,
                    )
                }
            }
}
