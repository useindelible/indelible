package app.indelible.share

import app.indelible.share.model.PendingItem
import app.indelible.share.repository.PendingSaveRepository

internal class FakePendingSaveRepository : PendingSaveRepository {
    val items = mutableListOf<PendingItem>()

    override suspend fun enqueue(item: PendingItem) {
        if (items.size < 50 && items.none { it.id == item.id }) {
            items.add(item)
        }
    }

    override suspend fun drainAll(): List<PendingItem> {
        val all = items.toList()
        items.clear()
        return all
    }

    override suspend fun remove(id: String) {
        items.removeAll { it.id == id }
    }
}
