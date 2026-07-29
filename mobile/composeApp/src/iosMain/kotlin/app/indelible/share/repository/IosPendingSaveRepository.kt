package app.indelible.share.repository

import app.indelible.share.model.PendingItem
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json
import platform.Foundation.NSUserDefaults

// The app group suite "group.com.useindelible.app" is used so that the iOS share
// extension (a separate process) can read and write the same defaults as the main app.
class IosPendingSaveRepository : PendingSaveRepository {
    private val defaults = NSUserDefaults(suiteName = APP_GROUP_SUITE)
    private val json = Json { ignoreUnknownKeys = true }

    override suspend fun enqueue(item: PendingItem) {
        val current = loadAll().toMutableList()
        if (current.size >= MAX_QUEUE_SIZE) return
        if (current.none { it.id == item.id }) {
            current.add(item)
            saveAll(current)
        }
    }

    override suspend fun drainAll(): List<PendingItem> {
        val items = loadAll()
        saveAll(emptyList())
        return items
    }

    override suspend fun remove(id: String) {
        val current = loadAll().filter { it.id != id }
        saveAll(current)
    }

    private fun loadAll(): List<PendingItem> {
        val raw = defaults?.stringForKey(KEY_PENDING_ITEMS) ?: return emptyList()
        return try {
            json.decodeFromString<List<PendingItem>>(raw)
        } catch (_: Exception) {
            emptyList()
        }
    }

    private fun saveAll(items: List<PendingItem>) {
        defaults?.setObject(json.encodeToString(items), KEY_PENDING_ITEMS)
    }

    companion object {
        private const val APP_GROUP_SUITE = "group.com.useindelible.app"
        private const val KEY_PENDING_ITEMS = "pending_items"
        private const val MAX_QUEUE_SIZE = 50
    }
}
