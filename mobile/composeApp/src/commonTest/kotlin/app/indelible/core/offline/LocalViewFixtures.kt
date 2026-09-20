package app.indelible.core.offline

import kotlinx.coroutines.flow.first

internal const val VIEW_DOC = "doc_1"
internal const val INSTALL_AT = 5_000L
internal const val REFRESH_AT = 9_000L

internal fun serverDocument(
    highlights: List<CachedHighlight> = emptyList(),
    note: String? = null,
    progress: Progress = Progress(percent = null, maxPercent = null),
    title: String = "Title",
): ServerDocument =
    ServerDocument(
        title = title,
        readerJson = """{"document_id":"$VIEW_DOC","title":"$title"}""",
        progress = progress,
        highlights = highlights,
        note = note?.let { ServerNote(it, updatedAtEpochMs = 1_000L) },
    )

internal fun installRequest(
    revision: DocumentRevision,
    server: ServerDocument = serverDocument(),
    generation: Long = 1,
    assets: List<CachedAssetRow> = listOf(CachedAssetRow(VIEW_DOC, "readable_html", 0, "readable.html", 10L)),
    pin: Boolean = false,
): InstallRequest =
    InstallRequest(
        documentId = VIEW_DOC,
        documentType = "article",
        revision = revision,
        server = server,
        generation = generation,
        assets = assets,
        bytes = assets.sumOf { it.bytes },
        pin = pin,
        at = INSTALL_AT,
    )

internal suspend fun SignedIn.revision(documentId: String = VIEW_DOC): DocumentRevision =
    store.localChanges(scope, documentId).revision

/** Installs against the revisions as they stand now, the way an uncontended download would. */
internal suspend fun SignedIn.installNow(
    server: ServerDocument = serverDocument(),
    pin: Boolean = false,
    generation: Long = 1,
): InstallResult {
    val request = installRequest(revision(), server, generation = generation, pin = pin)
    return store.installCachedDocument(session, request)
}

internal suspend fun SignedIn.enqueue(payload: OutboxPayload): String {
    val row = outboxRow(0, payload)
    store.enqueue(session, row.kind, row.entityId, VIEW_DOC) { payload to Unit }
    return store
        .observeOutbox(scope)
        .first()
        .last()
        .id
}

internal suspend fun SignedIn.enqueueProgress(basisPoints: Int?): String {
    store.enqueue(session, OutboxKind.READING_EVENT, VIEW_DOC, VIEW_DOC) {
        progressEvent(allocateOriginSeq(), basisPoints) to Unit
    }
    return store
        .observeOutbox(scope)
        .first()
        .last()
        .id
}

internal suspend fun SignedIn.row(id: String): OutboxRow = store.observeOutbox(scope).first().single { it.id == id }
