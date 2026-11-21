package app.indelible.core.offline

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

class FakeNetworkFailure : Exception("no network")

fun rejected(message: String) = SyncFailure(SyncFailure.Kind.REJECTED, serverMessage = message)

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
