package app.indelible.reader.repository

import app.indelible.api.generated.models.DocumentNoteResponse
import app.indelible.api.generated.models.DocumentReaderResponse
import app.indelible.api.generated.models.HighlightListResponse
import app.indelible.core.offline.CachedDocumentRow
import app.indelible.core.offline.DocumentRevision
import app.indelible.core.offline.LocalChanges
import app.indelible.core.offline.OfflineFiles
import app.indelible.core.offline.OfflineStore
import app.indelible.core.offline.Progress
import app.indelible.core.offline.RefreshRequest
import app.indelible.core.offline.ScopeNotLiveException
import app.indelible.core.offline.ServerNote
import app.indelible.core.offline.ServerPart
import app.indelible.core.offline.Session
import app.indelible.core.offline.StaleWriteException
import app.indelible.core.offline.generationDir
import app.indelible.core.offline.progress
import app.indelible.core.offline.toCachedHighlight
import app.indelible.reader.model.ArticleToc
import app.indelible.reader.model.ArticleTocStatus
import app.indelible.reader.model.EpubToc
import app.indelible.reader.model.HighlightData
import app.indelible.reader.model.ReaderDocument
import app.indelible.reader.model.toHighlightData
import app.indelible.reader.model.toReaderDocument
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import kotlinx.serialization.json.Json
import okio.IOException

/** A note or highlights read that a local change kept overtaking; nothing was shown, so retrying is safe. */
class StaleReadException : Exception("The document changed while it was being read")

/** A copy's asset of [kind]; [idx] null matches any index. */
private data class AssetSlot(
    val kind: String,
    val idx: Int? = null,
)

/** A server part fetched with the live rows read after it; [held] when no local change overtook the fetch. */
private class Steady<S>(
    val server: S,
    val changes: LocalChanges,
    val held: Boolean,
)

// Note and highlights save whole, so showing an overtaken value would let an edit overwrite that change.
private fun <S, V> Result<Steady<S>>.whenHeld(view: (Steady<S>) -> V): Result<V> =
    fold(
        onSuccess = { if (it.held) Result.success(view(it)) else Result.failure(StaleReadException()) },
        onFailure = { Result.failure(it) },
    )

/**
 * The local view: server parts with local changes replayed on top; a part a local change
 * overtook during its fetch is never applied, and a server slower than [FALLBACK_BUDGET_MS] falls back to the copy.
 */
class ReaderOfflineCopy(
    private val store: OfflineStore,
    private val files: OfflineFiles,
    private val sessionProvider: () -> Session?,
    private val clock: () -> Long,
    private val requestCache: suspend (Session, String) -> Unit,
    private val dispatcher: CoroutineDispatcher = Dispatchers.Default,
) {
    val tags = OfflineTags(files, store, sessionProvider, FALLBACK_BUDGET_MS, dispatcher)

    suspend fun document(
        documentId: String,
        fetch: suspend () -> Result<DocumentReaderResponse>,
    ): Result<ReaderDocument> {
        val session = sessionProvider() ?: return fetch().map { it.toReaderDocument() }
        store.touchDocumentOpened(session.scope, documentId, clock())
        val fromCopy =
            fromCopy(session, documentId, fetch, { it.readerPart() }) { copy ->
                json
                    .decodeFromString(DocumentReaderResponse.serializer(), copy.readerJson)
                    .toReaderDocument()
                    .withProgress(Progress(copy.progressPercent, copy.maxProgressPercent, copy.position))
            }
        // Progress is append-only; the server never lets an older event win, so an overtaken view still stands.
        return fromCopy ?: steady(session.scope, documentId, { it.position }, fetch)
            .map { it.server.toReaderDocument().withProgress(it.changes.progress(it.server.progress())) }
            .onSuccess { requestCache(session, documentId) }
    }

    suspend fun highlights(
        documentId: String,
        fetch: suspend () -> Result<HighlightListResponse>,
    ): Result<List<HighlightData>> {
        val session =
            sessionProvider() ?: return fetch().map { response -> response.highlights.map { it.toHighlightData() } }
        val fromCopy =
            fromCopy(session, documentId, fetch, { ServerPart.Highlights(it.toCached(documentId)) }) {
                store.cachedHighlights(session.scope, documentId).map { it.toHighlightData(clock()) }
            }
        return fromCopy ?: steady(session.scope, documentId, { it.content }, fetch).whenHeld { steady ->
            steady.changes.highlights(steady.server.toCached(documentId)).map { it.toHighlightData(clock()) }
        }
    }

    suspend fun note(
        documentId: String,
        fetch: suspend () -> Result<DocumentNoteResponse?>,
    ): Result<String?> {
        val session = sessionProvider() ?: return fetch().map { it?.body }
        val fromCopy = fromCopy(session, documentId, fetch, { ServerPart.Note(it?.toServerNote()) }) { it.noteBody }
        return fromCopy ?: steady(session.scope, documentId, { it.content }, fetch).whenHeld { steady ->
            steady.changes.note(steady.server?.body)
        }
    }

    suspend fun readableHtml(
        documentId: String,
        fetch: suspend () -> Result<String>,
    ): Result<String> = fileOrFetch(documentId, READABLE_HTML, fetch) { it }

    /** A copy without an outline answers "none", so an offline reader stops polling for one. */
    suspend fun articleToc(
        documentId: String,
        fetch: suspend () -> Result<ArticleToc>,
    ): Result<ArticleToc> =
        fileOrFetch(documentId, ARTICLE_TOC, fetch, whenAbsent = NO_TOC) {
            json.decodeFromString(ArticleToc.serializer(), it)
        }

    /** With a copy, the book's contents come from its file; the server only when that file is missing or unreadable. */
    suspend fun epubToc(
        documentId: String,
        fetch: suspend (String) -> Result<EpubToc>,
    ): Result<EpubToc> =
        copyFirst(documentId, AssetSlot(EPUB_TOC, 0), { fetch(documentId) }) {
            json.decodeFromString(EpubToc.serializer(), it)
        }

    /** The chapter at [spineIndex], from the copy first, as [epubToc]. */
    suspend fun epubChapter(
        documentId: String,
        spineIndex: Int,
        fetch: suspend () -> Result<String>,
    ): Result<String> = copyFirst(documentId, AssetSlot(EPUB_CHAPTER, spineIndex), fetch) { it }

    // Copy's view after refreshing with the fetched part, or null without a copy; an overtaken part is dropped.
    private suspend fun <S, V> fromCopy(
        session: Session,
        documentId: String,
        fetch: suspend () -> Result<S>,
        part: (S) -> ServerPart,
        read: suspend (CachedDocumentRow) -> V,
    ): Result<V>? {
        val scope = session.scope
        val revision = store.localChanges(scope, documentId).revision
        if (store.cachedDocument(scope, documentId) == null) return null
        val fetched = attemptWithin(FALLBACK_BUDGET_MS, fetch)
        fetched.onSuccess { refresh(session, RefreshRequest(documentId, revision, part(it), clock())) }
        val refused = fetched.exceptionOrNull()?.takeUnless { it.fallsBack() }
        val copy = store.cachedDocument(scope, documentId)
        return when {
            refused != null -> Result.failure(refused)
            copy == null -> null
            else -> Result.success(read(copy))
        }
    }

    private suspend fun refresh(
        session: Session,
        request: RefreshRequest,
    ) {
        try {
            store.refreshCachedCopy(session, request)
        } catch (ignored: StaleWriteException) {
            // The session ended mid-read; the reader it served is closing, so the copy's view stands.
        } catch (ignored: ScopeNotLiveException) {
            // The scope is being purged; the same holds.
        }
    }

    /** Retries until [part]'s revision holds across the fetch, up to [READ_ATTEMPTS] tries; last attempt returned. */
    private suspend fun <S> steady(
        scope: String,
        documentId: String,
        part: (DocumentRevision) -> Long,
        fetch: suspend () -> Result<S>,
    ): Result<Steady<S>> {
        var attempts = 0
        var result: Result<Steady<S>>
        do {
            val before = part(store.localChanges(scope, documentId).revision)
            result =
                fetch().map { server ->
                    val changes = store.localChanges(scope, documentId)
                    Steady(server, changes, held = part(changes.revision) == before)
                }
            attempts++
        } while (result.getOrNull()?.held == false && attempts < READ_ATTEMPTS)
        return result
    }

    /** A file part is the server's while it answers and the copy's when it cannot. */
    private suspend fun <V> fileOrFetch(
        documentId: String,
        kind: String,
        fetch: suspend () -> Result<V>,
        whenAbsent: V? = null,
        decode: (String) -> V,
    ): Result<V> {
        val session = sessionProvider()
        val copy = session?.let { store.cachedDocument(it.scope, documentId) }
        if (session == null || copy == null) return fetch()
        val fetched = attemptWithin(FALLBACK_BUDGET_MS, fetch)
        val fallback =
            fetched
                .exceptionOrNull()
                ?.takeIf { it.fallsBack() }
                ?.let { fromFile(session.scope, copy, AssetSlot(kind), whenAbsent, decode) }
        return fallback?.let { Result.success(it) } ?: fetched
    }

    private suspend fun <V> copyFirst(
        documentId: String,
        slot: AssetSlot,
        fetch: suspend () -> Result<V>,
        decode: (String) -> V,
    ): Result<V> {
        val session = sessionProvider()
        val copy = session?.let { store.cachedDocument(it.scope, documentId) }
        val local = copy?.let { fromFile(session.scope, it, slot, whenAbsent = null, decode) }
        return local?.let { Result.success(it) } ?: fetch().onFailure { if (it is CancellationException) throw it }
    }

    /** The copy's file in [slot]; [whenAbsent] without one, null when it cannot be read. */
    private suspend fun <V> fromFile(
        scope: String,
        copy: CachedDocumentRow,
        slot: AssetSlot,
        whenAbsent: V?,
        decode: (String) -> V,
    ): V? {
        val asset =
            store
                .assetsForDocument(scope, copy.documentId)
                .firstOrNull { it.kind == slot.kind && (slot.idx == null || it.idx == slot.idx) }
                ?: return whenAbsent
        val path = files.generationDir(scope, copy.documentId, copy.generation) / asset.path
        val text =
            withContext(dispatcher) {
                try {
                    files.readText(path)
                } catch (ignored: IOException) {
                    null
                }
            }
        return text?.let { runCatching { decode(it) }.getOrNull() }
    }

    private companion object {
        const val FALLBACK_BUDGET_MS = 3_000L
        const val READ_ATTEMPTS = 3
        const val READABLE_HTML = "readable_html"
        const val ARTICLE_TOC = "article_toc"
        const val EPUB_TOC = "epub_toc"
        const val EPUB_CHAPTER = "epub_chapter"

        val json = Json { ignoreUnknownKeys = true }
        val NO_TOC = ArticleToc(entries = emptyList(), status = ArticleTocStatus.NONE, truncated = false)

        fun DocumentReaderResponse.readerPart(): ServerPart =
            ServerPart.Reader(title, json.encodeToString(DocumentReaderResponse.serializer(), this), progress())

        fun ReaderDocument.withProgress(progress: Progress): ReaderDocument =
            copy(
                progressPercent = progress.percent,
                maxProgressPercent = progress.maxPercent,
                chapterLocator = progress.position?.chapter,
                chapterOffset = progress.position?.offset,
            )

        fun HighlightListResponse.toCached(documentId: String) = highlights.map { it.toCachedHighlight(documentId) }

        fun DocumentNoteResponse.toServerNote() = ServerNote(body, updatedAt.toEpochMilliseconds())
    }
}
