package app.indelible.core.offline

import app.indelible.core.network.AuthenticatedApiTransport
import app.indelible.core.storage.DEFAULT_OFFLINE_CAP_BYTES
import app.indelible.core.storage.InMemoryTokenStorage
import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.MockEngineConfig
import io.ktor.client.engine.mock.MockRequestHandleScope
import io.ktor.client.engine.mock.respond
import io.ktor.client.request.HttpRequestData
import io.ktor.client.request.HttpResponseData
import io.ktor.http.HttpHeaders
import io.ktor.http.HttpStatusCode
import io.ktor.http.headersOf
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.job
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.TestCoroutineScheduler
import kotlinx.coroutines.test.TestScope
import okio.FileSystem
import okio.Path.Companion.toPath
import okio.fakefilesystem.FakeFileSystem

internal const val DL_SCOPE = "http://a.test|u1"
internal const val DL_DOC = "doc_1"
private const val OFFLINE_ROOT = "/offline"
private const val FAR_FUTURE_EXPIRY = 4_102_444_800L
private const val WHEN = "2026-01-01T00:00:00Z"

internal class Call(
    val host: String,
    val path: String,
    val authorization: String?,
)

internal typealias Route = suspend MockRequestHandleScope.(HttpRequestData) -> HttpResponseData

/** Serves one document the way the API does; tests replace single routes to inject behaviour. */
internal class FakeServer {
    val calls = mutableListOf<Call>()
    val routes = mutableMapOf<String, Route>()
    var documentType = "article"
    var available = listOf("readable_html", "article_toc")

    /** Asset sizes the reader JSON announces, by asset kind. */
    var sizes = emptyMap<String, Long>()

    init {
        route("/api/v1/documents/$DL_DOC") { json(readerJson()) }
        route("/api/v1/documents/$DL_DOC/highlights") { json(HIGHLIGHTS) }
        route("/api/v1/documents/$DL_DOC/note") { json(NOTE) }
        route("/api/v1/assets/documents/$DL_DOC/readable_html") { respond("<p>hello</p>", HttpStatusCode.OK) }
        route("/api/v1/documents/$DL_DOC/toc") { json(ARTICLE_TOC) }
        route("/api/v1/assets/documents/$DL_DOC/pdf") {
            respond("", HttpStatusCode.Found, headersOf(HttpHeaders.Location, "http://s3.test/bucket/doc_1.pdf"))
        }
        route("/bucket/doc_1.pdf") { respond("%PDF-1.7", HttpStatusCode.OK) }
        route("/api/v1/documents/$DL_DOC/epub/toc") { json(EPUB_TOC) }
        for (index in 0..2) {
            route("/api/v1/documents/$DL_DOC/epub/chapters/$index") { respond("<h1>$index</h1>", HttpStatusCode.OK) }
        }
    }

    fun route(
        path: String,
        handler: Route,
    ) {
        routes[path] = handler
    }

    fun count(path: String): Int = calls.count { it.path == path }

    fun readerJson(): String {
        val available = available.joinToString(",") { "\"$it\"" }
        val assets =
            sizes.entries.joinToString(",") { (kind, size) ->
                """{"id":"ast_$kind","asset_kind":"$kind","content_type":"text/html","created_at":"$WHEN",""" +
                    """"size_bytes":$size,"status":"ready"}"""
            }
        return """{"document_id":"$DL_DOC","document_type":"$documentType","title":"Doc","saved":true,""" +
            """"readable_ready":true,"available_assets":[$available],"assets":[$assets],""" +
            """"progress_percent":20,"max_progress_percent":20}"""
    }
}

internal fun MockRequestHandleScope.json(body: String): HttpResponseData =
    respond(body, HttpStatusCode.OK, headersOf(HttpHeaders.ContentType, "application/json"))

private val HIGHLIGHTS =
    """{"count":1,"highlights":[{"id":"hlt_1","color":"yellow","text_content":"quoted","tags":["t"],""" +
        """"created_at":"$WHEN","updated_at":"$WHEN"}]}"""
private val NOTE = """{"id":"note_1","body":"server note","created_at":"$WHEN","updated_at":"$WHEN"}"""
private const val ARTICLE_TOC = """{"entries":[],"status":"none","truncated":false}"""
private val EPUB_TOC =
    """{"metadata":{"estimated_pages":3,"total_chapters":3,"total_words":30},"toc":[""" +
        listOf(0, 1, 1, 2)
            .mapIndexed { i, spine ->
                val depth = if (i == 2) 1 else 0
                """{"id":"e$i","title":"T$i","depth":$depth,"spine_index":$spine,"start_page":1,"word_count":10}"""
            }.joinToString(",") + "]}"

internal class DownloadHarness(
    val signedIn: SignedIn,
    val fs: FileSystem,
    val server: FakeServer,
    val transport: AuthenticatedApiTransport,
    backing: OfflineStore,
    scheduler: TestCoroutineScheduler,
) {
    var capBytes = DEFAULT_OFFLINE_CAP_BYTES
    var capSource: suspend () -> Long = { capBytes }
    val files = OfflineFiles(OFFLINE_ROOT.toPath(), fs)
    val online = MutableStateFlow(true)
    val copies = OfflineCopies(backing, files) { capSource() }
    val fetcher =
        OfflineSetFetcher(transport, backing, files, clock = { scheduler.currentTime }, capBytes = { capSource() })
    val manager =
        DownloadManager(signedIn.registry, fetcher, backing, copies, online, StandardTestDispatcher(scheduler))
    val store: SqlDelightOfflineStore get() = signedIn.store
    val session: Session get() = signedIn.session

    fun generationDir(generation: Long) = files.generationDir(DL_SCOPE, DL_DOC, generation)

    suspend fun acquisition(): Acquisition? = manager.acquisitions.observe(DL_SCOPE).first()[DL_DOC]
}

internal suspend fun TestScope.downloadHarness(
    fs: FileSystem = FakeFileSystem(),
    store: (SqlDelightOfflineStore) -> OfflineStore = { it },
    configure: FakeServer.() -> Unit = {},
): DownloadHarness {
    val signedIn = signedInTestStore(DL_SCOPE)
    val server = FakeServer().apply(configure)
    val engine =
        MockEngine(
            MockEngineConfig().apply {
                dispatcher = StandardTestDispatcher(testScheduler)
                addHandler { request ->
                    val path = request.url.encodedPath
                    server.calls += Call(request.url.host, path, request.headers[HttpHeaders.Authorization])
                    server.routes[path]?.invoke(this, request) ?: respond("", HttpStatusCode.NotFound)
                }
            },
        )
    val tokens =
        InMemoryTokenStorage().apply {
            saveToken("token")
            saveExpiresAt(FAR_FUTURE_EXPIRY)
            saveServerUrl("http://a.test")
        }
    val transport = AuthenticatedApiTransport(tokens, engine = engine, registry = signedIn.registry)
    val harness = DownloadHarness(signedIn, fs, server, transport, store(signedIn.store), testScheduler)
    harness.manager.start()
    backgroundScope.coroutineContext.job.invokeOnCompletion { harness.manager.stop() }
    return harness
}
