package app.indelible.reader.repository

import app.indelible.core.network.AuthenticatedApiTransport
import app.indelible.core.network.LibraryApiService
import app.indelible.core.network.ReaderApiService
import app.indelible.core.offline.CachedHighlightRow
import app.indelible.core.offline.EnqueueTx
import app.indelible.core.offline.FakeNetworkFailure
import app.indelible.core.offline.OfflineStore
import app.indelible.core.offline.OutboxKind
import app.indelible.core.offline.OutboxPayload
import app.indelible.core.offline.OutboxRow
import app.indelible.core.offline.OutboxSender
import app.indelible.core.offline.SendOutcome
import app.indelible.core.offline.SqlDelightOfflineStore
import app.indelible.core.offline.testWorker
import app.indelible.core.storage.InMemoryTokenStorage
import app.indelible.db.testOfflineDatabase
import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.respond
import io.ktor.http.HttpHeaders
import io.ktor.http.HttpMethod
import io.ktor.http.HttpStatusCode
import io.ktor.http.content.TextContent
import io.ktor.http.headersOf
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.withContext
import kotlinx.coroutines.withTimeout

/** Far enough ahead of the rows' real creation stamps that the worker considers them drainable. */
private const val DRAIN_CLOCK = 10_000_000_000_000L

internal const val OFFLINE_SCOPE = "http://localhost:38473|usr_1"
internal const val OFFLINE_DOCUMENT_ID = "doc_01"
private const val FAR_FUTURE_EXPIRY = 4_102_444_800L
private const val AWAIT_TIMEOUT_MS = 5_000L
private const val AWAIT_POLL_MS = 5L

/** Every send fails as a network error, so enqueued rows stay readable after the drain. */
internal class StuckOutboxSender : OutboxSender {
    var calls = 0
        private set

    override suspend fun send(
        scope: String,
        batch: List<OutboxRow>,
    ): SendOutcome {
        calls++
        return SendOutcome.Transport(FakeNetworkFailure())
    }
}

/** Stands in for a local write that cannot commit: a full disk, a corrupt database. */
internal class EnqueueFailure : Exception("offline store unavailable")

internal class ThrowingEnqueueStore(
    delegate: OfflineStore,
) : OfflineStore by delegate {
    override suspend fun <T> enqueue(
        scope: String,
        kind: OutboxKind,
        entityId: String,
        documentId: String,
        buildPayload: EnqueueTx.() -> Pair<OutboxPayload, T>,
    ): T = throw EnqueueFailure()
}

internal class RecordedRequest(
    val method: HttpMethod,
    val path: String,
    val body: String?,
)

internal class ReaderOutboxHarness(
    val store: OfflineStore,
    val sender: StuckOutboxSender,
    val repository: ApiReaderRepository,
    val requests: List<RecordedRequest>,
)

internal suspend fun readerOutboxHarness(
    drainScope: CoroutineScope,
    scope: String? = OFFLINE_SCOPE,
    failingStore: Boolean = false,
): ReaderOutboxHarness {
    val tokenStorage = InMemoryTokenStorage()
    tokenStorage.saveToken("test-token")
    tokenStorage.saveExpiresAt(FAR_FUTURE_EXPIRY)
    tokenStorage.saveServerUrl("http://localhost:38473")
    val requests = mutableListOf<RecordedRequest>()
    val engine =
        MockEngine { request ->
            requests += RecordedRequest(request.method, request.url.encodedPath, (request.body as? TextContent)?.text)
            respond("{}", HttpStatusCode.OK, headersOf(HttpHeaders.ContentType, "application/json"))
        }
    val transport = AuthenticatedApiTransport(tokenStorage, engine = engine)
    val backing = SqlDelightOfflineStore(testOfflineDatabase())
    val store = if (failingStore) ThrowingEnqueueStore(backing) else backing
    val sender = StuckOutboxSender()
    return ReaderOutboxHarness(
        store = store,
        sender = sender,
        repository =
            ApiReaderRepository(
                readerApiService = ReaderApiService(transport),
                libraryApiService = LibraryApiService(transport),
                offlineStore = store,
                worker = testWorker(store, sender, scope) { DRAIN_CLOCK },
                scopeProvider = { scope },
                drainScope = drainScope,
            ),
        requests = requests,
    )
}

/** Reads a cached highlight through an enqueue receiver, the only transactional read there is. */
internal suspend fun OfflineStore.probeCachedHighlight(
    scope: String,
    id: String,
): CachedHighlightRow? =
    enqueue(scope, OutboxKind.DOCUMENT_NOTE, "probe", "probe") {
        OutboxPayload.DocumentNote("probe", null) to getCachedHighlight(id)
    }

/** The store commits on [Dispatchers.Default], so a caller that did not await the write polls. */
internal suspend fun <T> awaitStore(block: suspend () -> T?): T =
    withContext(Dispatchers.Default) {
        withTimeout(AWAIT_TIMEOUT_MS) {
            while (true) {
                block()?.let { return@withTimeout it }
                delay(AWAIT_POLL_MS)
            }
            error("unreachable")
        }
    }

internal suspend fun OfflineStore.rowsOf(
    scope: String,
    kind: OutboxKind,
): List<OutboxRow> = drainable(scope, Long.MAX_VALUE).filter { it.kind == kind }
