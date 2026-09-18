package app.indelible.core.offline

import kotlinx.coroutines.flow.Flow

/** The queue of local writes waiting to reach the server, in seq order per scope. */
interface OutboxStore {
    /**
     * Runs [buildPayload]'s cache mutations and the outbox insert in one transaction, after
     * checking inside that transaction that [session] is still the published one
     * ([StaleWriteException]) and that its scope is live ([ScopeNotLiveException]).
     * The [EnqueueTx] receiver exposes `allocateOriginSeq()` so a reading event's seq is
     * claimed atomically with its row; the receiver is unusable once this call returns.
     */
    suspend fun <T> enqueue(
        session: Session,
        kind: OutboxKind,
        entityId: String,
        documentId: String,
        buildPayload: EnqueueTx.() -> Pair<OutboxPayload, T>,
    ): T

    /** Every unsuperseded PENDING row in seq order; due-ness is decided by the caller. */
    suspend fun pendingOrdered(scope: String): List<OutboxRow>

    suspend fun rowsByState(
        scope: String,
        state: OutboxState,
    ): List<OutboxRow>

    /**
     * Acknowledges the row in one transaction: deletes it, supersedes every older row that sets
     * the same field (all older edits of a highlight, for a delete), and moves the document's
     * revision. Reading events are never superseded; their acknowledgement raises the document's
     * acknowledged progress seq instead.
     */
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

    /**
     * Resets the row to PENDING with attempts 0 and unblocks BLOCKED rows for its entity, and
     * returns true. Refused, returning false, unless the row is FAILED and not superseded.
     */
    suspend fun retryRow(
        scope: String,
        id: String,
    ): Boolean
}

/** Live views of the queue for screens that show sync state. */
interface OutboxObservation {
    /** Seq-ordered, full rows including nextAttemptAt. */
    fun observeOutbox(scope: String): Flow<List<OutboxRow>>

    fun observeOutboxForDocument(
        scope: String,
        documentId: String,
    ): Flow<List<OutboxRow>>

    /** Counts per document id; documents without unsuperseded rows are absent. */
    fun observeDocumentSyncCounts(scope: String): Flow<Map<String, DocumentSyncCounts>>
}

/** Cached document metadata: pinning, recency and byte accounting for eviction. */
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

    fun observeCatalog(scope: String): Flow<List<CatalogEntry>>
}

/** Cached document content: assets, highlights and the sync stamp, always per document. */
interface CachedContentStore {
    /**
     * Installs a complete copy in one transaction, after the same liveness check as enqueue.
     * The note, highlights and progress written are the local view over [InstallRequest.server];
     * pinned and last-synced survive a reinstall. Writes nothing and returns [InstallResult.Stale]
     * when the document's revision moved since [InstallRequest.revision] was read.
     */
    suspend fun installCachedDocument(
        session: Session,
        request: InstallRequest,
    ): InstallResult

    /** Merges a server part into an existing copy, unless the revision it depends on moved. */
    suspend fun refreshCachedCopy(
        session: Session,
        request: RefreshRequest,
    ): RefreshResult

    /** The live rows and revision of one document, read in one transaction. */
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

    /** Sets cached_document.last_synced_at; no-op without a cached row. */
    suspend fun markDocumentSynced(
        scope: String,
        documentId: String,
        at: Long,
    )

    /**
     * Deletes the document's copy; never touches the outbox. Highlights that still have live
     * outbox rows stay, so a later change to them still has a local value to apply to.
     */
    suspend fun removeCachedDocument(
        scope: String,
        documentId: String,
    )

    /** Deletes cached highlights of documents without a copy once no live outbox row needs them. */
    suspend fun dropOrphanHighlights(scope: String)
}

/** Per-scope lifecycle: client identity, purge state and the write barrier transitions rely on. */
interface ScopeStore {
    /** Returns once every write that already held the store lock has committed. */
    suspend fun quiesce()

    /** Creates the client identity for scope if none exists; the only place that does. */
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

/** Everything the offline store offers; the parts are separate so a caller can depend on one. */
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

    /** Keeps a cached copy's note on the edit, so it survives its own acknowledgement. */
    fun setCachedNote(
        documentId: String,
        body: String,
    )

    /** Moves a cached copy's progress, raising its maximum; no-op without a copy. */
    fun patchCachedProgress(
        documentId: String,
        percent: Int,
    )
}
