package app.indelible.core.offline

import kotlinx.coroutines.flow.Flow

interface OutboxStore {
    /** Cache writes and outbox insert in one transaction; throws [StaleWriteException] or [ScopeNotLiveException]. */
    suspend fun <T> enqueue(
        session: Session,
        kind: OutboxKind,
        entityId: String,
        documentId: String,
        buildPayload: EnqueueTx.() -> Pair<OutboxPayload, T>,
    ): T

    /** Unsuperseded PENDING rows in seq order; the caller decides which are due. */
    suspend fun pendingOrdered(scope: String): List<OutboxRow>

    suspend fun rowsByState(
        scope: String,
        state: OutboxState,
    ): List<OutboxRow>

    /** Deletes the row, supersedes older same-field rows, moves the revision; never supersedes reading events. */
    suspend fun remove(
        scope: String,
        id: String,
    )

    suspend fun markAttempt(
        scope: String,
        id: String,
        now: Long,
        nextAttemptAt: Long,
        error: String?,
    )

    suspend fun markFailed(
        scope: String,
        id: String,
        error: String?,
    )

    /** One transaction: the create becomes FAILED and every PENDING row for its entity becomes BLOCKED. */
    suspend fun failCreateAndBlockDependants(
        scope: String,
        id: String,
        entityId: String,
        error: String?,
    )

    /** Resets the row to PENDING, unblocks its BLOCKED rows, and returns true; false unless FAILED and unsuperseded. */
    suspend fun retryRow(
        scope: String,
        id: String,
    ): Boolean
}

interface OutboxObservation {
    fun observeOutbox(scope: String): Flow<List<OutboxRow>>

    fun observeOutboxForDocument(
        scope: String,
        documentId: String,
    ): Flow<List<OutboxRow>>

    fun observeDocumentSyncCounts(scope: String): Flow<Map<String, DocumentSyncCounts>>
}

interface CachedDocumentStore {
    suspend fun upsertCachedDocument(
        scope: String,
        row: CachedDocumentRow,
    )

    suspend fun touchDocumentOpened(
        scope: String,
        documentId: String,
        at: Long,
    )

    /** False when there is no copy to pin or unpin. */
    suspend fun setPinned(
        scope: String,
        documentId: String,
        pinned: Boolean,
    ): Boolean

    suspend fun setDocumentBytes(
        scope: String,
        documentId: String,
        bytes: Long,
    )

    suspend fun cachedDocuments(scope: String): List<CachedDocumentRow>

    suspend fun cachedDocument(
        scope: String,
        documentId: String,
    ): CachedDocumentRow?

    suspend fun unpinnedLru(scope: String): List<CachedDocumentRow>

    suspend fun totalBytes(scope: String): Long

    fun observeCatalog(scope: String): Flow<List<CatalogEntry>>
}

interface CachedContentStore {
    /** Liveness-checked like enqueue; writes nothing and returns [InstallResult.Stale] if the revision moved. */
    suspend fun installCachedDocument(
        session: Session,
        request: InstallRequest,
    ): InstallResult

    /** Merges a server part into an existing copy, unless the revision it depends on moved. */
    suspend fun refreshCachedCopy(
        session: Session,
        request: RefreshRequest,
    ): RefreshResult

    suspend fun localChanges(
        scope: String,
        documentId: String,
    ): LocalChanges

    suspend fun cachedHighlights(
        scope: String,
        documentId: String,
    ): List<CachedHighlight>

    suspend fun assetsForDocument(
        scope: String,
        documentId: String,
    ): List<CachedAssetRow>

    suspend fun markDocumentSynced(
        scope: String,
        documentId: String,
        at: Long,
    )

    /** Deletes the document's copy; never touches the outbox, so highlights with live outbox rows stay. */
    suspend fun removeCachedDocument(
        scope: String,
        documentId: String,
    )

    /** Removes the copy as [removeCachedDocument] does, but only while it is unpinned; true if it did. */
    suspend fun evictCachedDocument(
        scope: String,
        documentId: String,
    ): Boolean

    suspend fun dropOrphanHighlights(scope: String)
}

interface ScopeStore {
    /** Returns once every write that already held the store lock has committed. */
    suspend fun quiesce()

    suspend fun clientIdentity(scope: String): ClientIdentity

    suspend fun profile(scope: String): String?

    suspend fun keepProfile(
        scope: String,
        json: String,
    )

    suspend fun scopesWithState(): List<Pair<String, Boolean>>

    suspend fun setPurgePending(
        scope: String,
        pending: Boolean,
    )

    /** Deletes cached_* and outbox rows for scope; client_state survives since it carries purge_pending. */
    suspend fun purgeRows(scope: String)

    /** Deletes the client_state row; must be the last purge step. */
    suspend fun finishPurge(scope: String)
}

interface OfflineStore :
    OutboxStore,
    OutboxObservation,
    CachedDocumentStore,
    CachedContentStore,
    ScopeStore

interface EnqueueTx {
    fun allocateOriginSeq(): Long

    fun getCachedHighlight(id: String): CachedHighlightRow?

    fun upsertCachedHighlight(
        id: String,
        documentId: String,
        payloadJson: String,
        updatedAt: Long,
    )

    fun deleteCachedHighlight(id: String)

    fun setCachedNote(
        documentId: String,
        body: String,
    )

    fun patchCachedProgress(
        documentId: String,
        percent: Int,
    )
}
