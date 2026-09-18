package app.indelible.core.offline

import app.indelible.db.OfflineDatabase
import app.indelible.db.OfflineQueries
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.coroutines.withContext
import kotlinx.serialization.json.Json

internal val offlineJson = Json { ignoreUnknownKeys = true }

/** The database, dispatcher, session registry and write lock every part of the SQLDelight store shares. */
internal class SqlDelightStoreContext(
    val database: OfflineDatabase,
    val dispatcher: CoroutineDispatcher,
    val registry: SessionRegistry,
) {
    val queries: OfflineQueries get() = database.offlineQueries

    // JdbcSqliteDriver in file mode (the desktop jvm() target) hands each thread its own SQLite
    // connection, so two transactionWithResult calls on different Dispatchers.Default threads do
    // not serialize against each other there the way they do on Android/iOS drivers. One
    // store-level lock around every write path keeps write ordering target-agnostic.
    private val mutex = Mutex()

    suspend fun <T> write(block: () -> T): T = mutex.withLock { withContext(dispatcher) { block() } }

    suspend fun <T> read(block: () -> T): T = withContext(dispatcher) { block() }

    /** Returns once every write that already held the lock has committed. */
    suspend fun quiesce() {
        mutex.withLock { }
    }

    /** Called inside a write transaction: [session] is still the published one and its scope is live. */
    fun requireLive(session: Session) {
        if (registry.current.value.session !== session) throw StaleWriteException()
        val state = queries.getClientState(session.scope).executeAsOneOrNull()
        if (state == null || state.purge_pending != 0L) throw ScopeNotLiveException(session.scope)
    }
}

class SqlDelightOfflineStore private constructor(
    context: SqlDelightStoreContext,
    failCreateHook: () -> Unit,
) : OfflineStore,
    OutboxStore by SqlDelightOutbox(context, failCreateHook),
    OutboxObservation by SqlDelightOutboxObservation(context),
    CachedDocumentStore by SqlDelightCachedDocuments(context),
    CachedContentStore by SqlDelightCachedContent(context),
    ScopeStore by SqlDelightScopes(context) {
    constructor(
        database: OfflineDatabase,
        failCreateHook: () -> Unit = {},
        dispatcher: CoroutineDispatcher = Dispatchers.Default,
        registry: SessionRegistry = SessionRegistry(),
    ) : this(SqlDelightStoreContext(database, dispatcher, registry), failCreateHook)
}
