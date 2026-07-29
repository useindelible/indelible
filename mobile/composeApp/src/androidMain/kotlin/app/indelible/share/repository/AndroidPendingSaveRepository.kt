package app.indelible.share.repository

import android.content.Context
import android.content.SharedPreferences
import androidx.security.crypto.EncryptedSharedPreferences
import androidx.security.crypto.MasterKeys
import app.indelible.share.model.PendingItem
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json

class AndroidPendingSaveRepository(
    context: Context,
) : PendingSaveRepository {
    private val prefs: SharedPreferences by lazy {
        val masterKeyAlias = MasterKeys.getOrCreate(MasterKeys.AES256_GCM_SPEC)
        EncryptedSharedPreferences.create(
            PREFS_NAME,
            masterKeyAlias,
            context,
            EncryptedSharedPreferences.PrefKeyEncryptionScheme.AES256_SIV,
            EncryptedSharedPreferences.PrefValueEncryptionScheme.AES256_GCM,
        )
    }

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
        val raw = prefs.getString(KEY_PENDING_ITEMS, null) ?: return emptyList()
        return try {
            json.decodeFromString<List<PendingItem>>(raw)
        } catch (_: Exception) {
            emptyList()
        }
    }

    private fun saveAll(items: List<PendingItem>) {
        prefs.edit().putString(KEY_PENDING_ITEMS, json.encodeToString(items)).apply()
    }

    companion object {
        private const val PREFS_NAME = "indelible_auth"
        private const val KEY_PENDING_ITEMS = "pending_items"
        private const val MAX_QUEUE_SIZE = 50
    }
}
