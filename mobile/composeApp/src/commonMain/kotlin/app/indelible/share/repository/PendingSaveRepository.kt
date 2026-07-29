package app.indelible.share.repository

import app.indelible.share.model.PendingItem

interface PendingSaveRepository {
    suspend fun enqueue(item: PendingItem)

    suspend fun drainAll(): List<PendingItem>

    suspend fun remove(id: String)
}
