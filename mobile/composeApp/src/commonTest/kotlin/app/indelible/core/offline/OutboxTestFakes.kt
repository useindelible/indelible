package app.indelible.core.offline

import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.first

class FakeOutboxSender : OutboxSender {
    val calls = mutableListOf<List<OutboxRow>>()
    private val outcomes = ArrayDeque<SendOutcome>()

    fun enqueueOutcome(outcome: SendOutcome) {
        outcomes.addLast(outcome)
    }

    override suspend fun send(
        session: Session,
        clientId: String,
        batch: List<OutboxRow>,
    ): SendOutcome {
        calls += batch
        return outcomes.removeFirstOrNull() ?: error("FakeOutboxSender: no outcome queued for call #${calls.size}")
    }
}

/** A never-started worker with no resolvable scope, so only resumeAuth() and authPaused are observable. */
fun testOutboxWorker(): OutboxWorker =
    OutboxWorker(
        UnreachableOfflineStore,
        SessionRegistry(),
        FakeOutboxSender(),
        clock = { 0L },
    )

/**
 * A purger backed by a store with no known scopes, so [ScopePurger.purgeInactive] and
 * [ScopePurger.resumeInterrupted] are no-ops regardless of the active scope passed in.
 */
fun testScopePurger(): ScopePurger = ScopePurger(NoScopesOfflineStore)

private object NoScopesOfflineStore : OfflineStore by UnreachableOfflineStore {
    override suspend fun scopesWithState(): List<Pair<String, Boolean>> = emptyList()
}

/**
 * Every member throws rather than returning a neutral value: a scope-less worker must never reach
 * the store, so any call here is a bug that should fail loudly instead of passing on empty data.
 */
private object UnreachableOfflineStore : OfflineStore {
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
        error: String?,
    ): Unit = unreachable()

    override suspend fun failCreateAndBlockDependants(
        scope: String,
        id: String,
        entityId: String,
        error: String?,
    ): Unit = unreachable()

    override suspend fun retryRow(
        scope: String,
        id: String,
    ): Boolean = unreachable()

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
    ): Unit = unreachable()

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

    override suspend fun clientIdentity(scope: String): ClientIdentity = unreachable()

    override suspend fun scopesWithState(): List<Pair<String, Boolean>> = unreachable()

    override suspend fun setPurgePending(
        scope: String,
        pending: Boolean,
    ): Unit = unreachable()

    override suspend fun purgeRows(scope: String): Unit = unreachable()

    override suspend fun finishPurge(scope: String): Unit = unreachable()
}

private fun unreachable(): Nothing = throw UnsupportedOperationException("offline store reached unexpectedly")

class FakeNetworkFailure : Exception("no network")

class MarkDocumentSyncedSpyStore(
    private val delegate: OfflineStore,
) : OfflineStore by delegate {
    var markDocumentSyncedCallCount = 0
        private set

    override suspend fun markDocumentSynced(
        scope: String,
        documentId: String,
        at: Long,
    ) {
        markDocumentSyncedCallCount++
        delegate.markDocumentSynced(scope, documentId, at)
    }
}

class FakeClock(
    var now: Long = 0L,
) {
    fun current(): Long = now
}

suspend fun OfflineStore.enqueueNote(
    session: Session,
    entityId: String,
    documentId: String = entityId,
): String {
    enqueue(session, OutboxKind.DOCUMENT_NOTE, entityId, documentId) {
        OutboxPayload.DocumentNote(entityId, null) to Unit
    }
    return observeOutbox(session.scope).first().last().id
}

suspend fun OfflineStore.enqueueHighlightCreate(
    session: Session,
    highlightId: String,
    documentId: String,
): String {
    enqueue(session, OutboxKind.HIGHLIGHT_CREATE, highlightId, documentId) {
        OutboxPayload.HighlightCreate(highlightId, "yellow", "quoted text", null, null) to Unit
    }
    return observeOutbox(session.scope).first().last().id
}

suspend fun OfflineStore.enqueueHighlightColor(
    session: Session,
    highlightId: String,
    documentId: String,
): String {
    enqueue(session, OutboxKind.HIGHLIGHT_COLOR, highlightId, documentId) {
        OutboxPayload.HighlightColor(highlightId, "blue") to Unit
    }
    return observeOutbox(session.scope).first().last().id
}

suspend fun OfflineStore.enqueueHighlightDelete(
    session: Session,
    highlightId: String,
    documentId: String,
): String {
    enqueue(session, OutboxKind.HIGHLIGHT_DELETE, highlightId, documentId) {
        OutboxPayload.HighlightDelete(highlightId) to Unit
    }
    return observeOutbox(session.scope).first().last().id
}

suspend fun OfflineStore.enqueueReadingEvent(
    session: Session,
    documentId: String,
    recordedAtEpochMs: Long,
): String {
    enqueue(session, OutboxKind.READING_EVENT, documentId, documentId) {
        val seq = allocateOriginSeq()
        OutboxPayload.ReadingEvent(
            eventId = "evt_$seq",
            originSeq = seq,
            kind = "progress",
            progressBasisPoints = 1000,
            cause = "scroll",
            sessionId = "ses_1",
            attempt = 0,
            positionJson = null,
            assetKind = "epub",
            activeMs = 1_000,
            recordedAtEpochMs = recordedAtEpochMs,
        ) to Unit
    }
    return observeOutbox(session.scope).first().last().id
}
