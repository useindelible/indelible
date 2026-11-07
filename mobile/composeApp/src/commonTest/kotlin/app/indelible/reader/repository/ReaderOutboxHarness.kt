package app.indelible.reader.repository

import app.indelible.core.network.AuthenticatedApiTransport
import app.indelible.core.network.LibraryApiService
import app.indelible.core.network.ReaderApiService
import app.indelible.core.offline.CachedHighlightRow
import app.indelible.core.offline.EnqueueTx
import app.indelible.core.offline.OfflineStore
import app.indelible.core.offline.OutboxKind
import app.indelible.core.offline.OutboxPayload
import app.indelible.core.offline.OutboxRow
import app.indelible.core.offline.OutboxSender
import app.indelible.core.offline.SendOutcome
import app.indelible.core.offline.Session
import app.indelible.core.offline.SessionRegistry
import app.indelible.core.offline.SessionState
import app.indelible.core.offline.SqlDelightOfflineStore
import app.indelible.core.offline.startedWorker
import app.indelible.core.offline.testSession
import app.indelible.core.storage.InMemoryTokenStorage
import app.indelible.db.testOfflineDatabase
import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.respond
import io.ktor.http.HttpHeaders
import io.ktor.http.HttpMethod
import io.ktor.http.HttpStatusCode
import io.ktor.http.content.TextContent
import io.ktor.http.headersOf
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.TestScope

internal const val OFFLINE_SCOPE = "http://localhost:38473|usr_1"
internal const val OFFLINE_DOCUMENT_ID = "doc_01"
private const val FAR_FUTURE_EXPIRY = 4_102_444_800L

/** Every send is rejected as unauthorized, so the worker pauses and enqueued rows stay readable. */
internal class StuckOutboxSender : OutboxSender {
    var calls = 0
        private set

    override suspend fun send(
        session: Session,
        clientId: String,
        batch: List<OutboxRow>,
    ): SendOutcome {
        calls++
        return SendOutcome.Http(status = 401, retryAfterSeconds = null, message = "unauthorized")
    }
}

/** Stands in for a local write that cannot commit: a full disk, a corrupt database. */
internal class EnqueueFailure : Exception("offline store unavailable")

internal class ThrowingEnqueueStore(
    delegate: OfflineStore,
) : OfflineStore by delegate {
    override suspend fun <T> enqueue(
        session: Session,
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
    val registry: SessionRegistry,
) {
    val session: Session get() = checkNotNull(registry.current.value.session)
}

internal suspend fun TestScope.readerOutboxHarness(
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
    val registry = SessionRegistry()
    val transport = AuthenticatedApiTransport(tokenStorage, engine = engine, registry = registry)
    val backing =
        SqlDelightOfflineStore(
            testOfflineDatabase(),
            registry = registry,
            dispatcher = StandardTestDispatcher(testScheduler),
        )
    if (scope != null) {
        registry.publish(SessionState(0, testSession(scope)))
        backing.clientIdentity(scope)
    }
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
                worker = startedWorker(store, sender, registry),
                sessionProvider = { registry.current.value.session },
            ),
        requests = requests,
        registry = registry,
    )
}

/** Reads a cached highlight through an enqueue receiver, the only transactional read there is. */
internal suspend fun OfflineStore.probeCachedHighlight(
    session: Session,
    id: String,
): CachedHighlightRow? =
    enqueue(session, OutboxKind.DOCUMENT_NOTE, "probe", "probe") {
        OutboxPayload.DocumentNote("probe", null) to getCachedHighlight(id)
    }

internal suspend fun OfflineStore.rowsOf(
    scope: String,
    kind: OutboxKind,
): List<OutboxRow> = observeOutbox(scope).first().filter { it.kind == kind }
