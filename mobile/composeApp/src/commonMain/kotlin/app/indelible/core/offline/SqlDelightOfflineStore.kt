package app.indelible.core.offline

import app.cash.sqldelight.coroutines.asFlow
import app.cash.sqldelight.coroutines.mapToList
import app.indelible.core.util.clientId
import app.indelible.core.util.uuidV7
import app.indelible.db.Cached_asset
import app.indelible.db.Cached_document
import app.indelible.db.OfflineDatabase
import app.indelible.db.OfflineQueries
import app.indelible.db.Outbox
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.coroutines.withContext
import kotlinx.datetime.Clock
import kotlinx.serialization.decodeFromString
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json

internal val offlineJson = Json { ignoreUnknownKeys = true }

class SqlDelightOfflineStore(
    private val database: OfflineDatabase,
    private val failCreateHook: () -> Unit = {},
    private val dispatcher: CoroutineDispatcher = Dispatchers.Default,
) : OfflineStore {
    private val queries: OfflineQueries get() = database.offlineQueries

    // JdbcSqliteDriver in file mode (the desktop jvm() target) hands each thread its own SQLite
    // connection, so two transactionWithResult calls on different Dispatchers.Default threads do
    // not serialize against each other there the way they do on Android/iOS drivers. One
    // store-level lock around every write path keeps write ordering target-agnostic.
    private val mutex = Mutex()

    override suspend fun <T> enqueue(
        scope: String,
        kind: OutboxKind,
        entityId: String,
        documentId: String,
        buildPayload: EnqueueTx.() -> Pair<OutboxPayload, T>,
    ): T =
        mutex.withLock {
            withContext(dispatcher) {
                val tx = EnqueueTxImpl(scope, queries)
                try {
                    database.transactionWithResult {
                        val (payload, result) = tx.buildPayload()
                        queries.insertOutbox(
                            scope = scope,
                            id = uuidV7(),
                            kind = kind.wireName(),
                            entity_id = entityId,
                            document_id = documentId,
                            payload_json = offlineJson.encodeToString(payload),
                            created_at = Clock.System.now().toEpochMilliseconds(),
                            state = initialState(scope, kind, entityId).wireName(),
                        )
                        result
                    }
                } finally {
                    tx.close()
                }
            }
        }

    private fun initialState(
        scope: String,
        kind: OutboxKind,
        entityId: String,
    ): OutboxState =
        if (kind.dependsOnCreate() && queries.hasFailedCreate(scope, entityId).executeAsOne()) {
            OutboxState.BLOCKED
        } else {
            OutboxState.PENDING
        }

    override suspend fun pendingOrdered(scope: String): List<OutboxRow> =
        withContext(dispatcher) {
            queries.pendingOrdered(scope).executeAsList().map(::outboxRowFrom)
        }

    override suspend fun rowsByState(
        scope: String,
        state: OutboxState,
    ): List<OutboxRow> =
        withContext(dispatcher) {
            queries.outboxByState(scope, state.wireName()).executeAsList().map(::outboxRowFrom)
        }

    override suspend fun remove(
        scope: String,
        id: String,
    ) {
        mutex.withLock {
            withContext(dispatcher) {
                queries.deleteOutboxRow(scope, id)
            }
        }
    }

    override suspend fun markAttempt(
        scope: String,
        id: String,
        now: Long,
        nextAttemptAt: Long,
        error: String?,
    ) {
        mutex.withLock {
            withContext(dispatcher) {
                queries.markAttempt(
                    last_attempt_at = now,
                    next_attempt_at = nextAttemptAt,
                    last_error = error,
                    scope = scope,
                    id = id,
                )
            }
        }
    }

    override suspend fun markFailed(
        scope: String,
        id: String,
        error: String?,
    ) {
        mutex.withLock {
            withContext(dispatcher) {
                queries.setState(OutboxState.FAILED.wireName(), error, scope, id)
            }
        }
    }

    override suspend fun failCreateAndBlockDependants(
        scope: String,
        id: String,
        entityId: String,
        error: String?,
    ) {
        mutex.withLock {
            withContext(dispatcher) {
                database.transaction {
                    queries.setState(OutboxState.FAILED.wireName(), error, scope, id)
                    failCreateHook()
                    queries.setStateForEntity(
                        OutboxState.BLOCKED.wireName(),
                        scope,
                        entityId,
                        OutboxState.PENDING.wireName(),
                    )
                }
            }
        }
    }

    override suspend fun retryRow(
        scope: String,
        id: String,
    ) {
        mutex.withLock {
            withContext(dispatcher) {
                database.transaction {
                    val row = queries.outboxRow(scope, id).executeAsOneOrNull()
                    if (row != null && row.state == OutboxState.FAILED.wireName()) {
                        queries.resetForRetry(scope, id)
                        queries.setStateForEntity(
                            OutboxState.PENDING.wireName(),
                            scope,
                            row.entity_id,
                            OutboxState.BLOCKED.wireName(),
                        )
                    }
                }
            }
        }
    }

    override fun observeOutbox(scope: String): Flow<List<OutboxRow>> =
        queries
            .outboxForScope(scope)
            .asFlow()
            .mapToList(dispatcher)
            .map { rows -> rows.map(::outboxRowFrom) }

    override fun observeOutboxForDocument(
        scope: String,
        documentId: String,
    ): Flow<List<OutboxRow>> =
        queries.outboxForDocument(scope, documentId).asFlow().mapToList(dispatcher).map { rows ->
            rows.map(::outboxRowFrom)
        }

    override suspend fun upsertCachedDocument(
        scope: String,
        row: CachedDocumentRow,
    ) {
        mutex.withLock {
            withContext(dispatcher) {
                queries.upsertCachedDocument(
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
    }

    override suspend fun touchDocumentOpened(
        scope: String,
        documentId: String,
        at: Long,
    ) {
        mutex.withLock {
            withContext(dispatcher) {
                queries.touchDocumentOpened(at, scope, documentId)
            }
        }
    }

    override suspend fun setPinned(
        scope: String,
        documentId: String,
        pinned: Boolean,
    ) {
        mutex.withLock {
            withContext(dispatcher) {
                queries.setPinned(pinned.toLong(), scope, documentId)
            }
        }
    }

    override suspend fun setDocumentBytes(
        scope: String,
        documentId: String,
        bytes: Long,
    ) {
        mutex.withLock {
            withContext(dispatcher) {
                queries.setDocumentBytes(bytes, scope, documentId)
            }
        }
    }

    override suspend fun cachedDocuments(scope: String): List<CachedDocumentRow> =
        withContext(dispatcher) {
            queries.cachedDocuments(scope).executeAsList().map(::cachedDocumentRowFrom)
        }

    override suspend fun cachedDocument(
        scope: String,
        documentId: String,
    ): CachedDocumentRow? =
        withContext(dispatcher) {
            queries.cachedDocument(scope, documentId).executeAsOneOrNull()?.let(::cachedDocumentRowFrom)
        }

    override suspend fun unpinnedLru(scope: String): List<CachedDocumentRow> =
        withContext(dispatcher) {
            queries.unpinnedLru(scope).executeAsList().map(::cachedDocumentRowFrom)
        }

    override suspend fun totalBytes(scope: String): Long =
        withContext(dispatcher) {
            queries.totalBytes(scope).executeAsOne().SUM ?: 0L
        }

    override suspend fun installCachedDocument(
        scope: String,
        row: CachedDocumentRow,
        assets: List<CachedAssetRow>,
    ) {
        mutex.withLock {
            withContext(dispatcher) {
                database.transaction {
                    queries.upsertCachedDocument(
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
                    queries.deleteAssetsForDocument(scope, row.documentId)
                    assets.forEach { asset ->
                        queries.insertCachedAsset(
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
    }

    override suspend fun assetsForDocument(
        scope: String,
        documentId: String,
    ): List<CachedAssetRow> =
        withContext(dispatcher) {
            queries.assetsForDocument(scope, documentId).executeAsList().map(::cachedAssetRowFrom)
        }

    override suspend fun markDocumentSynced(
        scope: String,
        documentId: String,
        at: Long,
    ) {
        mutex.withLock {
            withContext(dispatcher) {
                queries.setDocumentSyncedAt(at, scope, documentId)
            }
        }
    }

    override suspend fun upsertCachedHighlight(
        scope: String,
        id: String,
        documentId: String,
        payloadJson: String,
        updatedAt: Long,
    ) {
        mutex.withLock {
            withContext(dispatcher) {
                queries.upsertCachedHighlight(scope, id, documentId, payloadJson, updatedAt)
            }
        }
    }

    override suspend fun removeCachedDocument(
        scope: String,
        documentId: String,
    ) {
        mutex.withLock {
            withContext(dispatcher) {
                database.transaction {
                    queries.deleteCachedDocument(scope, documentId)
                    queries.deleteAssetsForDocument(scope, documentId)
                    queries.deleteHighlightsForDocument(scope, documentId)
                }
            }
        }
    }

    override suspend fun clientIdentity(scope: String): ClientIdentity =
        mutex.withLock {
            withContext(dispatcher) {
                database.transactionWithResult {
                    val existing = queries.getClientState(scope).executeAsOneOrNull()
                    if (existing != null) {
                        ClientIdentity(clientId = existing.client_id, purgePending = existing.purge_pending != 0L)
                    } else {
                        val newClientId = clientId()
                        queries.insertClientState(scope, newClientId, 0, 0)
                        ClientIdentity(clientId = newClientId, purgePending = false)
                    }
                }
            }
        }

    override suspend fun scopesWithState(): List<Pair<String, Boolean>> =
        withContext(dispatcher) {
            queries.scopesWithState().executeAsList().map { it.scope to (it.purge_pending != 0L) }
        }

    override suspend fun setPurgePending(
        scope: String,
        pending: Boolean,
    ) {
        mutex.withLock {
            withContext(dispatcher) {
                queries.setPurgePending(pending.toLong(), scope)
            }
        }
    }

    override suspend fun purgeRows(scope: String) {
        mutex.withLock {
            withContext(dispatcher) {
                queries.purgeRows(scope)
            }
        }
    }

    override suspend fun finishPurge(scope: String) {
        mutex.withLock {
            withContext(dispatcher) {
                queries.deleteClientState(scope)
            }
        }
    }
}

private fun OutboxKind.wireName(): String = name.lowercase()

private fun OutboxKind.dependsOnCreate(): Boolean =
    when (this) {
        OutboxKind.HIGHLIGHT_COLOR,
        OutboxKind.HIGHLIGHT_NOTE,
        OutboxKind.HIGHLIGHT_TAGS,
        OutboxKind.HIGHLIGHT_DELETE,
        -> true
        OutboxKind.READING_EVENT,
        OutboxKind.HIGHLIGHT_CREATE,
        OutboxKind.DOCUMENT_NOTE,
        -> false
    }

private fun OutboxState.wireName(): String = name.lowercase()

private fun Boolean.toLong(): Long = if (this) 1L else 0L

private fun outboxRowFrom(row: Outbox): OutboxRow =
    OutboxRow(
        seq = row.seq,
        scope = row.scope,
        id = row.id,
        kind = OutboxKind.valueOf(row.kind.uppercase()),
        entityId = row.entity_id,
        documentId = row.document_id,
        payload = offlineJson.decodeFromString(row.payload_json),
        createdAt = row.created_at,
        attempts = row.attempts.toInt(),
        lastAttemptAt = row.last_attempt_at,
        nextAttemptAt = row.next_attempt_at,
        state = OutboxState.valueOf(row.state.uppercase()),
        lastError = row.last_error,
    )

private fun cachedDocumentRowFrom(row: Cached_document): CachedDocumentRow =
    CachedDocumentRow(
        documentId = row.document_id,
        documentType = row.document_type,
        title = row.title,
        readerJson = row.reader_json,
        pinned = row.pinned != 0L,
        lastOpenedAt = row.last_opened_at,
        lastSyncedAt = row.last_synced_at,
        bytes = row.bytes,
    )

private fun cachedAssetRowFrom(row: Cached_asset): CachedAssetRow =
    CachedAssetRow(
        documentId = row.document_id,
        kind = row.kind,
        idx = row.idx.toInt(),
        path = row.path,
        bytes = row.bytes,
    )
