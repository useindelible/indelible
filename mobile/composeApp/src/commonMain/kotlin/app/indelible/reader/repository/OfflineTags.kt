package app.indelible.reader.repository

import app.indelible.core.offline.OfflineFiles
import app.indelible.core.offline.OfflineStore
import app.indelible.core.offline.OutboxPayload
import app.indelible.core.offline.Session
import app.indelible.core.offline.scopeDir
import app.indelible.reader.model.TagData
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.coroutines.withContext
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.SerializationException
import kotlinx.serialization.builtins.ListSerializer
import kotlinx.serialization.json.Json
import okio.IOException

private const val TAGS_FILE = "tags.json"
private const val LOCAL_ID_PREFIX = "local:"

@Serializable
private data class KeptTag(
    val id: String,
    val name: String,
    @SerialName("highlight_count") val highlightCount: Long = 0,
)

private val keptTagList = ListSerializer(KeptTag.serializer())
private val json = Json { ignoreUnknownKeys = true }

/**
 * The account's last tag list, kept beside its offline copies so the tag sheet still offers it
 * without the server. A list that cannot be read or written only costs those suggestions.
 */
class OfflineTags internal constructor(
    private val files: OfflineFiles,
    private val store: OfflineStore,
    private val sessionProvider: () -> Session?,
    private val budgetMs: Long,
    private val dispatcher: CoroutineDispatcher,
) {
    private val lock = Mutex()

    /** The server's tag list, or the one it last gave when it cannot answer. */
    suspend fun list(fetch: suspend () -> Result<List<TagData>>): Result<List<TagData>> {
        val session = sessionProvider() ?: return fetch()
        val fetched = attemptWithin(budgetMs, fetch).map { replace(session, it) }
        val kept = fetched.exceptionOrNull()?.takeIf { it.fallsBack() }?.let { read(session.scope) }
        return kept?.let { Result.success(it) } ?: fetched
    }

    /** Adds the [names] the list lacks, so a tag first used without the server is offered again. */
    suspend fun keep(
        session: Session,
        names: List<String>,
    ) = lock.withLock {
        val kept = load(session.scope).orEmpty()
        val known = kept.mapTo(mutableSetOf()) { it.name }
        val added = names.distinct().filter { it !in known }.map { KeptTag("$LOCAL_ID_PREFIX$it", it) }
        if (added.isNotEmpty()) write(session, kept + added)
    }

    /** Returns once no write is in flight; a later write sees the ended session and skips. */
    suspend fun quiesce() = lock.withLock { }

    private suspend fun read(scope: String): List<TagData>? =
        lock.withLock { load(scope) }?.map { TagData(id = it.id, name = it.name, highlightCount = it.highlightCount) }

    private suspend fun replace(
        session: Session,
        tags: List<TagData>,
    ): List<TagData> =
        lock.withLock {
            val listed = tags.mapTo(mutableSetOf()) { it.name }
            val queued = queuedNames(session.scope)
            val pending =
                load(session.scope).orEmpty().filter {
                    it.id.startsWith(LOCAL_ID_PREFIX) && it.name !in listed && it.name in queued
                }
            write(session, tags.map { KeptTag(it.id, it.name, it.highlightCount) } + pending)
            tags + pending.map { TagData(id = it.id, name = it.name, highlightCount = it.highlightCount) }
        }

    private suspend fun queuedNames(scope: String): Set<String> =
        store
            .observeOutbox(scope)
            .first()
            .flatMapTo(mutableSetOf()) { (it.payload as? OutboxPayload.HighlightTags)?.tags.orEmpty() }

    private suspend fun load(scope: String): List<KeptTag>? =
        withContext(dispatcher) {
            try {
                files.readText(files.scopeDir(scope) / TAGS_FILE)?.let { json.decodeFromString(keptTagList, it) }
            } catch (ignored: IOException) {
                null
            } catch (ignored: SerializationException) {
                null
            }
        }

    private suspend fun write(
        session: Session,
        tags: List<KeptTag>,
    ) = withContext(dispatcher) {
        if (sessionProvider() !== session) return@withContext
        try {
            files.writeText(files.scopeDir(session.scope) / TAGS_FILE, json.encodeToString(keptTagList, tags))
        } catch (ignored: IOException) {
        }
    }
}
