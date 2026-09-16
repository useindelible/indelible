package app.indelible.core.offline

import kotlinx.coroutines.flow.Flow

interface OfflineStore {
    /**
     * Runs [buildPayload]'s cache mutations and the outbox insert in one transaction.
     * The [EnqueueTx] receiver exposes `allocateOriginSeq()` so a reading event's seq is
     * claimed atomically with its row; the receiver is unusable once this call returns.
     */
    suspend fun <T> enqueue(
        scope: String,
        kind: OutboxKind,
        entityId: String,
        documentId: String,
        buildPayload: EnqueueTx.() -> Pair<OutboxPayload, T>,
    ): T

    suspend fun drainable(
        scope: String,
        now: Long,
    ): List<OutboxRow>

    suspend fun rowsByState(
        scope: String,
        state: OutboxState,
    ): List<OutboxRow>

    suspend fun earliestRetryAt(
        scope: String,
        now: Long,
    ): Long?

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

    /** Moves PENDING rows for the entity to BLOCKED. */
    suspend fun blockDependants(
        scope: String,
        entityId: String,
    )

    /**
     * Resets the row to PENDING with attempts 0 and unblocks BLOCKED rows for its entity.
     * No-op unless the target row is currently FAILED.
     */
    suspend fun retryRow(
        scope: String,
        id: String,
    )

    /** Seq-ordered, full rows including nextAttemptAt. */
    fun observeOutbox(scope: String): Flow<List<OutboxRow>>

    fun observeOutboxForDocument(
        scope: String,
        documentId: String,
    ): Flow<List<OutboxRow>>

    suspend fun upsertCachedDocument(
        scope: String,
        row: CachedDocumentRow,
    )

    suspend fun touchDocumentOpened(
        scope: String,
        documentId: String,
        at: Long,
    )

    suspend fun setPinned(
        scope: String,
        documentId: String,
        pinned: Boolean,
    )

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

    /** One transaction: upsert the document, delete its old asset rows, insert the new ones. */
    suspend fun installCachedDocument(
        scope: String,
        row: CachedDocumentRow,
        assets: List<CachedAssetRow>,
    )

    suspend fun assetsForDocument(
        scope: String,
        documentId: String,
    ): List<CachedAssetRow>

    /** Sets cached_document.last_synced_at; no-op without a cached row. */
    suspend fun markDocumentSynced(
        scope: String,
        documentId: String,
        at: Long,
    )

    suspend fun upsertCachedHighlight(
        scope: String,
        id: String,
        documentId: String,
        payloadJson: String,
        updatedAt: Long,
    )

    /** Deletes cached_* rows for the document only; never touches the outbox. */
    suspend fun removeCachedDocument(
        scope: String,
        documentId: String,
    )

    /** Lazily creates the client identity for scope via clientId() if none exists. */
    suspend fun clientIdentity(scope: String): ClientIdentity

    /** (scope, purgePending) for every known scope. */
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
}
