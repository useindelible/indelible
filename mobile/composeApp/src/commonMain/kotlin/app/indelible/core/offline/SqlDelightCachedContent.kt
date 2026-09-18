package app.indelible.core.offline

internal class SqlDelightCachedContent(
    private val context: SqlDelightStoreContext,
) : CachedContentStore {
    private val queries get() = context.queries

    override suspend fun installCachedDocument(
        session: Session,
        request: InstallRequest,
    ): InstallResult =
        context.write {
            context.database.transactionWithResult {
                context.requireLive(session)
                val stale = queries.staleness(session.scope, request.documentId, request.revision)
                if (stale.content || stale.position) {
                    InstallResult.Stale(content = stale.content, position = stale.position)
                } else {
                    install(session.scope, request)
                    InstallResult.Installed
                }
            }
        }

    override suspend fun refreshCachedCopy(
        session: Session,
        request: RefreshRequest,
    ): RefreshResult =
        context.write {
            context.database.transactionWithResult {
                context.requireLive(session)
                if (queries.cachedDocument(session.scope, request.documentId).executeAsOneOrNull() == null) {
                    RefreshResult.NoCopy
                } else {
                    RefreshResult.Applied(refresh(session.scope, request))
                }
            }
        }

    override suspend fun localChanges(
        scope: String,
        documentId: String,
    ): LocalChanges =
        context.read {
            context.database.transactionWithResult { queries.localChanges(scope, documentId) }
        }

    override suspend fun cachedHighlights(
        scope: String,
        documentId: String,
    ): List<CachedHighlight> =
        context.read {
            queries
                .highlightsForDocument(scope, documentId)
                .executeAsList()
                .map { offlineJson.decodeFromString(CachedHighlight.serializer(), it.payload_json) }
        }

    override suspend fun assetsForDocument(
        scope: String,
        documentId: String,
    ): List<CachedAssetRow> =
        context.read {
            queries
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
            queries.setDocumentSyncedAt(at, scope, documentId)
        }
    }

    override suspend fun removeCachedDocument(
        scope: String,
        documentId: String,
    ) {
        context.write {
            context.database.transaction {
                queries.deleteCachedDocument(scope, documentId)
                queries.deleteAssetsForDocument(scope, documentId)
                queries.deleteHighlightsWithoutLiveRows(scope = scope, document_id = documentId)
            }
        }
    }

    override suspend fun dropOrphanHighlights(scope: String) {
        context.write {
            queries.deleteOrphanHighlights(scope)
        }
    }

    private fun install(
        scope: String,
        request: InstallRequest,
    ) {
        val changes = queries.localChanges(scope, request.documentId)
        val existing = queries.cachedDocument(scope, request.documentId).executeAsOneOrNull()
        val server = request.server
        val progress = changes.progress(server.progress)
        queries.installDocument(
            scope = scope,
            document_id = request.documentId,
            document_type = request.documentType,
            title = server.title,
            reader_json = server.readerJson,
            pinned = (existing?.pinned == 1L || request.pin).toLong(),
            last_opened_at = request.at,
            last_synced_at = existing?.last_synced_at,
            bytes = request.bytes,
            generation = request.generation,
            note_body = changes.note(server.note?.body),
            note_server_updated_at = server.note?.updatedAtEpochMs,
            progress_percent = progress.percent?.toLong(),
            max_progress_percent = progress.maxPercent?.toLong(),
        )
        queries.deleteAssetsForDocument(scope, request.documentId)
        request.assets.forEach { asset ->
            queries.insertCachedAsset(scope, asset.documentId, asset.kind, asset.idx.toLong(), asset.path, asset.bytes)
        }
        replaceHighlights(scope, request.documentId, changes.highlights(server.highlights))
    }

    private fun refresh(
        scope: String,
        request: RefreshRequest,
    ): Staleness {
        val stale = queries.staleness(scope, request.documentId, request.revision)
        val changes = queries.localChanges(scope, request.documentId)
        val server = request.server
        queries.refreshDocumentMeta(server.title, server.readerJson, scope, request.documentId)
        if (!stale.position) {
            val progress = changes.progress(server.progress)
            queries.setCachedProgress(
                progress_percent = progress.percent?.toLong(),
                max_progress_percent = progress.maxPercent?.toLong(),
                scope = scope,
                document_id = request.documentId,
            )
        }
        if (!stale.content) {
            val note = changes.note(server.note?.body)
            queries.setCachedServerNote(note, server.note?.updatedAtEpochMs, scope, request.documentId)
            replaceHighlights(scope, request.documentId, changes.highlights(server.highlights))
        }
        return stale
    }

    private fun replaceHighlights(
        scope: String,
        documentId: String,
        highlights: List<CachedHighlight>,
    ) {
        queries.deleteHighlightsForDocument(scope, documentId)
        highlights.forEach { highlight ->
            queries.upsertCachedHighlight(
                scope,
                highlight.id,
                documentId,
                offlineJson.encodeToString(CachedHighlight.serializer(), highlight),
                highlight.updatedAtEpochMs ?: 0L,
            )
        }
    }
}
