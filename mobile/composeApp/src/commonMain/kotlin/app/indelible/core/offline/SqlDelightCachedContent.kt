package app.indelible.core.offline

internal class SqlDelightCachedContent(
    private val context: SqlDelightStoreContext,
) : CachedContentStore {
    override suspend fun installCachedDocument(
        scope: String,
        row: CachedDocumentRow,
        assets: List<CachedAssetRow>,
    ) {
        context.write {
            context.database.transaction {
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
                context.queries.deleteAssetsForDocument(scope, row.documentId)
                assets.forEach { asset ->
                    context.queries.insertCachedAsset(
                        scope,
                        asset.documentId,
                        asset.kind,
                        asset.idx.toLong(),
                        asset.path,
                        asset.bytes,
                    )
                }
            }
        }
    }

    override suspend fun assetsForDocument(
        scope: String,
        documentId: String,
    ): List<CachedAssetRow> =
        context.read {
            context.queries
                .assetsForDocument(scope, documentId)
                .executeAsList()
                .map(::cachedAssetRowFrom)
        }

    override suspend fun markDocumentSynced(
        scope: String,
        documentId: String,
        at: Long,
    ) {
        context.write {
            context.queries.setDocumentSyncedAt(at, scope, documentId)
        }
    }

    override suspend fun upsertCachedHighlight(
        scope: String,
        id: String,
        documentId: String,
        payloadJson: String,
        updatedAt: Long,
    ) {
        context.write {
            context.queries.upsertCachedHighlight(scope, id, documentId, payloadJson, updatedAt)
        }
    }

    override suspend fun removeCachedDocument(
        scope: String,
        documentId: String,
    ) {
        context.write {
            context.database.transaction {
                context.queries.deleteCachedDocument(scope, documentId)
                context.queries.deleteAssetsForDocument(scope, documentId)
                context.queries.deleteHighlightsForDocument(scope, documentId)
            }
        }
    }
}
