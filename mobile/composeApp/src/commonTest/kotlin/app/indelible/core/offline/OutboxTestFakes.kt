package app.indelible.core.offline

class FakeOutboxSender : OutboxSender {
    val calls = mutableListOf<List<OutboxRow>>()
    private val outcomes = ArrayDeque<SendOutcome>()

    fun enqueueOutcome(outcome: SendOutcome) {
        outcomes.addLast(outcome)
    }

    override suspend fun send(
        scope: String,
        batch: List<OutboxRow>,
    ): SendOutcome {
        calls += batch
        return outcomes.removeFirstOrNull() ?: error("FakeOutboxSender: no outcome queued for call #${calls.size}")
    }
}

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

fun testWorker(
    store: OfflineStore,
    sender: OutboxSender,
    scope: String?,
    clock: () -> Long,
): OutboxWorker = OutboxWorker(store, { scope }, sender, clock, isNetwork = { it is FakeNetworkFailure })

suspend fun OfflineStore.enqueueNote(
    scope: String,
    entityId: String,
    documentId: String = entityId,
): String {
    enqueue(scope, OutboxKind.DOCUMENT_NOTE, entityId, documentId) {
        OutboxPayload.DocumentNote(entityId, null) to Unit
    }
    return drainable(scope, Long.MAX_VALUE).last().id
}

suspend fun OfflineStore.enqueueHighlightCreate(
    scope: String,
    highlightId: String,
    documentId: String,
): String {
    enqueue(scope, OutboxKind.HIGHLIGHT_CREATE, highlightId, documentId) {
        OutboxPayload.HighlightCreate(highlightId, "yellow", "quoted text", null, null) to Unit
    }
    return drainable(scope, Long.MAX_VALUE).last().id
}

suspend fun OfflineStore.enqueueHighlightColor(
    scope: String,
    highlightId: String,
    documentId: String,
): String {
    enqueue(scope, OutboxKind.HIGHLIGHT_COLOR, highlightId, documentId) {
        OutboxPayload.HighlightColor(highlightId, "blue") to Unit
    }
    return drainable(scope, Long.MAX_VALUE).last().id
}

suspend fun OfflineStore.enqueueHighlightDelete(
    scope: String,
    highlightId: String,
    documentId: String,
): String {
    enqueue(scope, OutboxKind.HIGHLIGHT_DELETE, highlightId, documentId) {
        OutboxPayload.HighlightDelete(highlightId) to Unit
    }
    return drainable(scope, Long.MAX_VALUE).last().id
}

suspend fun OfflineStore.enqueueReadingEvent(
    scope: String,
    documentId: String,
    recordedAtEpochMs: Long,
): String {
    enqueue(scope, OutboxKind.READING_EVENT, documentId, documentId) {
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
    return drainable(scope, Long.MAX_VALUE).last().id
}
