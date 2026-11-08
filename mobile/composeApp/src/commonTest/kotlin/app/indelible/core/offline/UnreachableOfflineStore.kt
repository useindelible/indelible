package app.indelible.core.offline

import kotlinx.coroutines.flow.Flow

internal object NoScopesOfflineStore : OfflineStore by UnreachableOfflineStore {
    override suspend fun scopesWithState(): List<Pair<String, Boolean>> = emptyList()
}

/**
 * Every member throws rather than returning a neutral value: a scope-less worker must never reach
 * the store, so any call here is a bug that should fail loudly instead of passing on empty data.
 */
internal object UnreachableOfflineStore : OfflineStore {
    override suspend fun <T> enqueue(
        session: Session,
        kind: OutboxKind,
        entityId: String,
        documentId: String,
        buildPayload: EnqueueTx.() -> Pair<OutboxPayload, T>,
    ): T = unreachable()

    override suspend fun quiesce(): Unit = unreachable()

    override suspend fun pendingOrdered(scope: String): List<OutboxRow> = unreachable()

    override suspend fun rowsByState(
        scope: String,
        state: OutboxState,
    ): List<OutboxRow> = unreachable()

    override suspend fun remove(
        scope: String,
        id: String,
        ackedNoteUpdatedAt: Long?,
    ): Unit = unreachable()

    override suspend fun markAttempt(
        scope: String,
        id: String,
        now: Long,
        nextAttemptAt: Long,
        error: String?,
    ): Unit = unreachable()

    override suspend fun markFailed(
        scope: String,
        id: String,
        failure: SyncFailure,
    ): Unit = unreachable()

    override suspend fun failCreateAndBlockDependants(
        scope: String,
        id: String,
        entityId: String,
        failure: SyncFailure,
    ): Unit = unreachable()

    override suspend fun retryRow(
        scope: String,
        id: String,
    ): Boolean = unreachable()

    override suspend fun discardRows(
        session: Session,
        rowIds: List<String>,
    ): Int = unreachable()

    override suspend fun keepLocalNote(
        session: Session,
        rowId: String,
        seenServerUpdatedAt: Long,
    ): Boolean = unreachable()

    override suspend fun keepServerNote(
        session: Session,
        rowId: String,
        seenServerUpdatedAt: Long,
    ): Boolean = unreachable()

    override fun observeWaitingByScope(): Flow<Map<String, Int>> = unreachable()

    override fun observeOutbox(scope: String): Flow<List<OutboxRow>> = unreachable()

    override fun observeOutboxForDocument(
        scope: String,
        documentId: String,
    ): Flow<List<OutboxRow>> = unreachable()

    override fun observeDocumentSyncCounts(scope: String): Flow<Map<String, DocumentSyncCounts>> = unreachable()

    override suspend fun upsertCachedDocument(
        scope: String,
        row: CachedDocumentRow,
    ): Unit = unreachable()

    override suspend fun touchDocumentOpened(
        scope: String,
        documentId: String,
        at: Long,
    ): Unit = unreachable()

    override suspend fun setPinned(
        scope: String,
        documentId: String,
        pinned: Boolean,
    ): Boolean = unreachable()

    override suspend fun setDocumentBytes(
        scope: String,
        documentId: String,
        bytes: Long,
    ): Unit = unreachable()

    override suspend fun cachedDocuments(scope: String): List<CachedDocumentRow> = unreachable()

    override suspend fun cachedDocument(
        scope: String,
        documentId: String,
    ): CachedDocumentRow? = unreachable()

    override suspend fun unpinnedLru(scope: String): List<CachedDocumentRow> = unreachable()

    override suspend fun totalBytes(scope: String): Long = unreachable()

    override fun observeCatalog(scope: String): Flow<List<CatalogEntry>> = unreachable()

    override suspend fun installCachedDocument(
        session: Session,
        request: InstallRequest,
    ): InstallResult = unreachable()

    override suspend fun refreshCachedCopy(
        session: Session,
        request: RefreshRequest,
    ): RefreshResult = unreachable()

    override suspend fun localChanges(
        scope: String,
        documentId: String,
    ): LocalChanges = unreachable()

    override suspend fun cachedHighlights(
        scope: String,
        documentId: String,
    ): List<CachedHighlight> = unreachable()

    override suspend fun assetsForDocument(
        scope: String,
        documentId: String,
    ): List<CachedAssetRow> = unreachable()

    override suspend fun markDocumentSynced(
        scope: String,
        documentId: String,
        at: Long,
    ): Unit = unreachable()

    override suspend fun removeCachedDocument(
        scope: String,
        documentId: String,
    ): Unit = unreachable()

    override suspend fun evictCachedDocument(
        scope: String,
        documentId: String,
    ): Boolean = unreachable()

    override suspend fun dropOrphanHighlights(scope: String): Unit = unreachable()

    override suspend fun clientIdentity(scope: String): ClientIdentity = unreachable()

    override suspend fun profile(scope: String): String? = unreachable()

    override suspend fun keepProfile(
        scope: String,
        json: String,
    ): Unit = unreachable()

    override suspend fun scopesWithState(): List<Pair<String, Boolean>> = unreachable()

    override suspend fun setPurgePending(
        scope: String,
        pending: Boolean,
    ): Unit = unreachable()

    override suspend fun purgeRows(scope: String): Unit = unreachable()

    override suspend fun finishPurge(scope: String): Unit = unreachable()

    override suspend fun replaceList(
        session: Session,
        key: String,
        rows: List<SnapshotRow>,
        at: Long,
    ): Unit = unreachable()

    override suspend fun appendList(
        session: Session,
        key: String,
        rows: List<SnapshotRow>,
        at: Long,
    ): Unit = unreachable()

    override suspend fun list(
        scope: String,
        key: String,
    ): List<SnapshotRow> = unreachable()

    override fun observeList(
        scope: String,
        key: String,
    ): Flow<KeptList> = unreachable()

    override suspend fun lastSeen(
        scope: String,
        documentId: String,
    ): SnapshotRow? = unreachable()

    override suspend fun forget(
        session: Session,
        id: String,
        keep: (String) -> Boolean,
    ): Unit = unreachable()

    override suspend fun setTriageState(
        session: Session,
        documentId: String,
        state: String,
    ): Unit = unreachable()
}

private fun unreachable(): Nothing = throw UnsupportedOperationException("offline store reached unexpectedly")
