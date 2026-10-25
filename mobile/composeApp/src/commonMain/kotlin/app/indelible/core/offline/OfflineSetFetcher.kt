package app.indelible.core.offline

import app.indelible.api.generated.models.DocumentNoteResponse
import app.indelible.api.generated.models.DocumentReaderResponse
import app.indelible.api.generated.models.EpubChapterList
import app.indelible.api.generated.models.EpubTocResponse
import app.indelible.api.generated.models.HighlightListResponse
import app.indelible.core.network.ApiException
import app.indelible.core.network.AuthenticatedApiTransport
import app.indelible.core.network.EPUB_CHAPTER_LIST
import app.indelible.core.network.ServerCapabilities
import io.ktor.client.call.body
import io.ktor.client.request.HttpRequestBuilder
import io.ktor.client.request.get
import io.ktor.client.request.header
import io.ktor.client.request.prepareGet
import io.ktor.client.statement.HttpResponse
import io.ktor.client.statement.bodyAsChannel
import io.ktor.client.statement.bodyAsText
import io.ktor.http.HttpHeaders
import io.ktor.http.contentLength
import io.ktor.http.isSuccess
import kotlinx.coroutines.NonCancellable
import kotlinx.coroutines.withContext
import kotlinx.serialization.json.Json
import okio.IOException
import okio.Path

sealed interface FetchResult {
    data object Installed : FetchResult

    /** An auto-cache of a document with nothing readable yet or larger than the cap; nothing was installed. */
    data object Skipped : FetchResult

    /** Local changes moved under every attempt's fetch; nothing was installed. */
    data object Stale : FetchResult
}

// The same preference the web reader uses when a PDF was stored under more than one kind.
internal val PDF_KINDS = listOf("pdf", "original_upload")

/** The server keeps no chapter list for a book, so no copy of it could be known complete. */
open class ChapterListUnavailableException(
    message: String,
) : IllegalStateException(message)

/** The list is still being built; the server resolves this by itself, so a kept book waits for it. */
class ChapterListPendingException : ChapterListUnavailableException("the chapter list is pending")

/** The server predates the chapter list, so it can serve no book offline until it is updated. */
class ServerTooOldException : ChapterListUnavailableException("the server does not list book chapters")

/** The server could not say whether it lists book chapters; the download retries like a dropped connection. */
class CapabilitiesUnknownException : Exception("server capabilities could not be read")

/**
 * Fetches one document's offline set bound to a session and installs it. Files stream into a
 * fresh generation directory first; the reader JSON, highlights and note are fetched after the
 * document's revision is read, so an install that a local change overtook is refused and retried
 * with fresh parts rather than written.
 */
class OfflineSetFetcher(
    private val transport: AuthenticatedApiTransport,
    private val capabilities: ServerCapabilities,
    private val store: OfflineStore,
    private val files: OfflineFiles,
    private val clock: () -> Long,
    private val capBytes: suspend () -> Long,
) {
    private class Reader(
        val raw: String,
        val model: DocumentReaderResponse,
    )

    suspend fun download(
        session: Session,
        documentId: String,
        pin: Boolean,
        ignoreCap: Boolean = false,
        onProgress: (Float) -> Unit = {},
    ): FetchResult {
        val first = readerDocument(session, documentId)
        val uncapped = pin || ignoreCap
        if (!first.model.worthCaching(uncapped, capBytes())) return FetchResult.Skipped
        val scope = session.scope
        val previous = store.cachedDocument(scope, documentId)?.generation
        val generation = (previous ?: 0L) + 1
        val dir = files.generationDir(scope, documentId, generation)
        files.deleteTree(dir)
        try {
            val assets = downloadAssets(session, first.model, dir, onProgress)
            val bytes = assets.sumOf { it.bytes }
            val tooLarge = !uncapped && bytes > capBytes()
            val installed =
                !tooLarge &&
                    (1..INSTALL_ATTEMPTS).any {
                        val revision = store.localChanges(scope, documentId).revision
                        val request =
                            InstallRequest(
                                documentId = documentId,
                                documentType = first.model.documentType,
                                revision = revision,
                                server = serverDocument(session, documentId),
                                generation = generation,
                                assets = assets,
                                bytes = bytes,
                                pin = pin,
                                at = clock(),
                            )
                        store.installCachedDocument(session, request) == InstallResult.Installed
                    }
            if (installed) previous?.let { deleteQuietly(files.generationDir(scope, documentId, it)) }
            return when {
                tooLarge -> FetchResult.Skipped
                installed -> FetchResult.Installed
                else -> FetchResult.Stale
            }
        } finally {
            withContext(NonCancellable) { releaseUnlessInstalled(scope, documentId, generation, dir) }
        }
    }

    // The generation belongs to this download until its install commits, then to the catalog: a
    // cancellation that lands after the commit must not delete the files the new row points at.
    private suspend fun releaseUnlessInstalled(
        scope: String,
        documentId: String,
        generation: Long,
        dir: Path,
    ) {
        val installed = runCatching { store.cachedDocument(scope, documentId)?.generation == generation }
        if (installed.getOrDefault(true) == false) deleteQuietly(dir)
    }

    private fun deleteQuietly(dir: Path) {
        try {
            files.deleteTree(dir)
        } catch (ignored: IOException) {
            // The launch sweep removes any generation the catalog does not reference.
        }
    }

    private suspend fun downloadAssets(
        session: Session,
        model: DocumentReaderResponse,
        dir: Path,
        onProgress: (Float) -> Unit,
    ): List<CachedAssetRow> {
        val documentId = model.documentId
        val available = model.availableAssets
        return when (model.documentType) {
            TYPE_PDF -> {
                val kind = PDF_KINDS.firstOrNull { it in available } ?: return emptyList()
                val pdf = AssetFetch(kind, 0, assetRoute(documentId, kind), PDF_FILE)
                val announced = model.assets.firstOrNull { it.assetKind == kind }?.sizeBytes
                listOf(fetchAsset(session, documentId, pdf, dir, ByteProgress(announced, onProgress)))
            }
            TYPE_BOOK -> if (EPUB in available) bookAssets(session, documentId, dir, onProgress) else emptyList()
            else ->
                listOfNotNull(
                    AssetFetch(READABLE_HTML, 0, assetRoute(documentId, READABLE_HTML), HTML_FILE)
                        .takeIf { READABLE_HTML in available },
                    AssetFetch(ARTICLE_TOC, 0, "/api/v1/documents/$documentId/toc", ARTICLE_TOC_FILE)
                        .takeIf { ARTICLE_TOC in available },
                ).mapIndexed { index, fetch ->
                    fetchAsset(session, documentId, fetch, dir).also { onProgress((index + 1f) / 2) }
                }
        }
    }

    private suspend fun bookAssets(
        session: Session,
        documentId: String,
        dir: Path,
        onProgress: (Float) -> Unit,
    ): List<CachedAssetRow> {
        requireChapterListSupport(session)
        val tocFetch = AssetFetch(EPUB_TOC, 0, "/api/v1/documents/$documentId/epub/toc", EPUB_TOC_FILE)
        val toc = fetchAsset(session, documentId, tocFetch, dir)
        val tocJson = checkNotNull(files.readText(dir / toc.path))
        val outline = json.decodeFromString(EpubTocResponse.serializer(), tocJson)
        val listed =
            outline.metadata.chapters?.takeIf { outline.chapterList == EpubChapterList.READY }
                ?: throw when (outline.chapterList) {
                    EpubChapterList.PENDING -> ChapterListPendingException()
                    else -> ChapterListUnavailableException("the chapter list is ${outline.chapterList}")
                }
        val chapters =
            listed.mapIndexed { done, chapter ->
                val index = chapter.spineIndex
                val route = "/api/v1/documents/$documentId/epub/chapters/$index"
                val fetch = AssetFetch(EPUB_CHAPTER, index, route, "chapters/$index.html")
                fetchAsset(session, documentId, fetch, dir).also { onProgress((done + 1f) / listed.size) }
            }
        return listOf(toc) + chapters
    }

    private suspend fun requireChapterListSupport(session: Session) {
        when (capabilities.supports(session, EPUB_CHAPTER_LIST)) {
            true -> Unit
            false -> throw ServerTooOldException()
            null -> throw CapabilitiesUnknownException()
        }
    }

    private suspend fun fetchAsset(
        session: Session,
        documentId: String,
        fetch: AssetFetch,
        dir: Path,
        progress: ByteProgress? = null,
    ): CachedAssetRow {
        val bytes =
            transport.sessionRequest(session) { client, baseUrl, token ->
                client.prepareGet(baseUrl + fetch.route) { bearer(token) }.execute { response ->
                    response.requireSuccess()
                    val expected = response.contentLength()?.takeIf { it > 0 } ?: progress?.announced
                    val total =
                        files.write(dir / fetch.file, response.bodyAsChannel()) { written ->
                            if (progress != null && expected != null && expected > 0) progress.report(written, expected)
                        }
                    progress?.finish()
                    total
                }
            }
        return CachedAssetRow(documentId, fetch.kind, fetch.idx, fetch.file, bytes)
    }

    /** Reports bytes written over the expected total, clamped to 1 and only when it has moved by [PROGRESS_STEP]. */
    private class ByteProgress(
        val announced: Long?,
        private val onProgress: (Float) -> Unit,
    ) {
        private var last = -1f

        fun report(
            written: Long,
            expected: Long,
        ) {
            val fraction = minOf(1f, written.toFloat() / expected)
            if (fraction - last >= PROGRESS_STEP || (fraction == 1f && last < 1f)) {
                last = fraction
                onProgress(fraction)
            }
        }

        fun finish() {
            if (last >= 0f && last < 1f) {
                last = 1f
                onProgress(1f)
            }
        }
    }

    private suspend fun serverDocument(
        session: Session,
        documentId: String,
    ): ServerDocument {
        val reader = readerDocument(session, documentId)
        val highlights = decoded<HighlightListResponse>(session, "/api/v1/documents/$documentId/highlights")
        val note = note(session, documentId)
        return ServerDocument(
            title = reader.model.title,
            readerJson = reader.raw,
            progress = reader.model.progress(),
            highlights = highlights.highlights.map { it.toCachedHighlight(documentId) },
            note = note?.let { ServerNote(it.body, it.updatedAt.toEpochMilliseconds()) },
        )
    }

    private suspend fun note(
        session: Session,
        documentId: String,
    ): DocumentNoteResponse? =
        try {
            decoded<DocumentNoteResponse>(session, "/api/v1/documents/$documentId/note")
        } catch (error: ApiException) {
            if (error.statusCode == NOT_FOUND) null else throw error
        }

    private suspend fun readerDocument(
        session: Session,
        documentId: String,
    ): Reader {
        val raw =
            transport.sessionRequest(session) { client, baseUrl, token ->
                val response = client.get("$baseUrl/api/v1/documents/$documentId") { bearer(token) }
                response.requireSuccess()
                response.bodyAsText()
            }
        return Reader(raw, json.decodeFromString(DocumentReaderResponse.serializer(), raw))
    }

    private suspend inline fun <reified T> decoded(
        session: Session,
        route: String,
    ): T =
        transport.sessionRequest(session) { client, baseUrl, token ->
            client.get(baseUrl + route) { bearer(token) }.also { it.requireSuccess() }.body<T>()
        }

    private class AssetFetch(
        val kind: String,
        val idx: Int,
        val route: String,
        val file: String,
    )

    private companion object {
        const val INSTALL_ATTEMPTS = 3
        const val PROGRESS_STEP = 0.01f
        const val NOT_FOUND = 404
        const val ERROR_PREVIEW_CHARS = 200
        const val TYPE_PDF = "pdf"
        const val TYPE_BOOK = "book"
        const val READABLE_HTML = "readable_html"
        const val ARTICLE_TOC = "article_toc"
        const val EPUB = "epub"
        const val EPUB_TOC = "epub_toc"
        const val EPUB_CHAPTER = "epub_chapter"
        const val HTML_FILE = "readable.html"
        const val ARTICLE_TOC_FILE = "article_toc.json"
        const val EPUB_TOC_FILE = "epub_toc.json"
        const val PDF_FILE = "document.pdf"

        val json = Json { ignoreUnknownKeys = true }

        fun assetRoute(
            documentId: String,
            kind: String,
        ) = "/api/v1/assets/documents/$documentId/$kind"

        fun DocumentReaderResponse.readableKind(): String? =
            when (documentType) {
                TYPE_PDF -> PDF_KINDS.firstOrNull { it in availableAssets }
                TYPE_BOOK -> EPUB
                else -> READABLE_HTML
            }

        // A document the server has nothing readable for yet would install as a copy the reader
        // cannot open, so it is never one, pinned or not. The announced size is the stored asset's;
        // a book's chapters can unpack to more than its EPUB, so the downloaded total is checked
        // again before an auto-cache installs.
        fun DocumentReaderResponse.worthCaching(
            uncapped: Boolean,
            cap: Long,
        ): Boolean {
            val kind = readableKind()?.takeIf { it in availableAssets } ?: return false
            return uncapped || assets.filter { it.assetKind == kind }.sumOf { it.sizeBytes } <= cap
        }

        fun HttpRequestBuilder.bearer(token: String) {
            header(HttpHeaders.Authorization, "Bearer $token")
        }

        suspend fun HttpResponse.requireSuccess() {
            if (!status.isSuccess()) throw ApiException(status.value, bodyAsText().take(ERROR_PREVIEW_CHARS))
        }
    }
}
