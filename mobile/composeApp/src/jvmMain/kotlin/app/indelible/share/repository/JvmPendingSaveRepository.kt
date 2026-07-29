package app.indelible.share.repository

import app.indelible.share.model.PendingItem
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.serialization.Serializable
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json
import java.util.prefs.Preferences

@Serializable
data class PendingSaveQueueOwner(
    val serverUrl: String,
    val userId: String,
)

@Serializable
private data class PendingSaveQueueEnvelope(
    val owner: PendingSaveQueueOwner,
    val items: List<PendingItem>,
)

class JvmPendingSaveRepository(
    private val preferences: Preferences = Preferences.userNodeForPackage(JvmPendingSaveRepository::class.java),
    private val ownerProvider: suspend () -> PendingSaveQueueOwner?,
) : PendingSaveRepository {
    private val json = Json { ignoreUnknownKeys = true }
    private val mutex = Mutex()

    override suspend fun enqueue(item: PendingItem) =
        mutex.withLock {
            val owner = currentOwner() ?: return@withLock
            val envelope = loadEnvelope()
            val items = if (envelope?.owner == owner) envelope.items.toMutableList() else mutableListOf()
            if (items.size >= MAX_QUEUE_SIZE || items.any { it.id == item.id }) return@withLock

            items.add(item)
            saveEnvelope(PendingSaveQueueEnvelope(owner, items))
        }

    override suspend fun drainAll(): List<PendingItem> =
        mutex.withLock {
            val owner = currentOwner() ?: return@withLock emptyList()
            val envelope = loadEnvelope() ?: return@withLock emptyList()
            if (envelope.owner != owner) return@withLock emptyList()

            if (saveEnvelope(envelope.copy(items = emptyList()))) envelope.items else emptyList()
        }

    override suspend fun remove(id: String) =
        mutex.withLock {
            val owner = currentOwner() ?: return@withLock
            val envelope = loadEnvelope() ?: return@withLock
            if (envelope.owner != owner) return@withLock

            saveEnvelope(envelope.copy(items = envelope.items.filter { it.id != id }))
        }

    suspend fun clearAll() =
        mutex.withLock {
            try {
                preferences.remove(KEY_PENDING_QUEUE)
                preferences.flush()
            } catch (error: Exception) {
                reportStorageFailure("clear", error)
            }
        }

    private suspend fun currentOwner(): PendingSaveQueueOwner? =
        try {
            ownerProvider()
        } catch (error: Exception) {
            reportStorageFailure("resolve owner", error)
            null
        }

    private fun loadEnvelope(): PendingSaveQueueEnvelope? =
        try {
            val raw = preferences.get(KEY_PENDING_QUEUE, null) ?: return null
            json.decodeFromString<PendingSaveQueueEnvelope>(raw)
        } catch (error: Exception) {
            reportStorageFailure("read", error)
            null
        }

    private fun saveEnvelope(envelope: PendingSaveQueueEnvelope): Boolean =
        try {
            preferences.put(KEY_PENDING_QUEUE, json.encodeToString(envelope))
            preferences.flush()
            true
        } catch (error: Exception) {
            reportStorageFailure("write", error)
            false
        }

    private fun reportStorageFailure(
        operation: String,
        error: Exception,
    ) {
        System.err.println("Pending save storage $operation failed: ${error.message}")
    }

    private companion object {
        const val KEY_PENDING_QUEUE = "pending_items"
        const val MAX_QUEUE_SIZE = 50
    }
}
