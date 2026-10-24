package app.indelible.core.offline

import app.indelible.core.storage.InMemoryTokenStorage
import app.indelible.core.storage.TokenStorage
import app.indelible.db.OfflineDatabase
import app.indelible.db.testOfflineDatabase
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.TestScope

fun testSession(
    scope: String,
    epoch: Long = 0,
): Session = Session(epoch = epoch, origin = scope.substringBefore('|'), scope = scope)

/** A store whose registry has [session] published and whose scope has a live client_state row. */
data class SignedIn(
    val registry: SessionRegistry,
    val store: SqlDelightOfflineStore,
    val session: Session,
) {
    val scope: String get() = session.scope

    /** Publishes a session for another scope at the same epoch and makes that scope live. */
    suspend fun switchTo(scope: String): Session {
        val next = testSession(scope, registry.current.value.epoch)
        registry.publish(SessionState(next.epoch, next))
        store.clientIdentity(scope)
        return next
    }
}

suspend fun signedInStore(
    scope: String = "scope",
    database: OfflineDatabase = testOfflineDatabase(),
    dispatcher: CoroutineDispatcher = Dispatchers.Default,
    registry: SessionRegistry = SessionRegistry(),
): SignedIn {
    val session = testSession(scope, registry.current.value.epoch)
    registry.publish(SessionState(session.epoch, session))
    val store = SqlDelightOfflineStore(database, registry = registry, dispatcher = dispatcher)
    store.clientIdentity(scope)
    return SignedIn(registry, store, session)
}

suspend fun TestScope.signedInTestStore(
    scope: String = "scope",
    database: OfflineDatabase = testOfflineDatabase(),
): SignedIn = signedInStore(scope, database, StandardTestDispatcher(testScheduler))

class TransitionsHarness(
    val registry: SessionRegistry,
    val tokenStorage: InMemoryTokenStorage,
    val store: SqlDelightOfflineStore,
    val worker: OutboxWorker,
    val sender: GatedSender,
    val sessions: SessionTransitions,
) {
    fun currentScope(): String? {
        val session = registry.current.value.session
        return session?.scope
    }

    suspend fun signIn(
        serverUrl: String,
        userId: String,
        expectedEpoch: Long = sessions.epoch(),
    ): Session {
        sessions.transition(expectedEpoch) {
            tokenStorage.saveServerUrl(serverUrl)
            tokenStorage.saveToken("token-$userId")
            tokenStorage.saveUserId(userId)
        }
        return checkNotNull(registry.current.value.session) { "sign-in published no session" }
    }
}

/** A store whose scope identity cannot be created, standing in for a full or corrupt database. */
class FailingIdentityStore(
    delegate: OfflineStore,
) : OfflineStore by delegate {
    override suspend fun clientIdentity(scope: String): ClientIdentity = error("client_state unavailable")
}

fun TestScope.transitionsHarness(): TransitionsHarness {
    val registry = SessionRegistry()
    val tokenStorage = InMemoryTokenStorage()
    val store =
        SqlDelightOfflineStore(
            testOfflineDatabase(),
            registry = registry,
            dispatcher = StandardTestDispatcher(testScheduler),
        )
    val sender = GatedSender()
    val worker = startedWorker(store, sender, registry)
    val sessions = SessionTransitions(registry, tokenStorage, store, worker)
    return TransitionsHarness(registry, tokenStorage, store, worker, sender, sessions)
}

/** Transitions over a real store and a never-started worker, for view-model tests. */
fun testSessionTransitions(
    tokenStorage: TokenStorage,
    registry: SessionRegistry = SessionRegistry(),
    store: SqlDelightOfflineStore = SqlDelightOfflineStore(testOfflineDatabase(), registry = registry),
    worker: OutboxWorker = testOutboxWorker(),
): SessionTransitions = SessionTransitions(registry, tokenStorage, store, worker)
