package app.indelible.sidebar.viewmodel

import app.indelible.sidebar.model.Collection
import app.indelible.sidebar.model.SmartList
import app.indelible.sidebar.repository.SidebarRepository
import kotlinx.datetime.Instant
import kotlinx.serialization.json.JsonObject

class FakeSidebarRepository(
    var collectionsResult: Result<List<Collection>> = Result.success(emptyList()),
    var smartListsResult: Result<List<SmartList>> = Result.success(emptyList()),
) : SidebarRepository {
    var listCollectionsCallCount = 0
    var listSmartListsCallCount = 0

    override suspend fun listCollections(): Result<List<Collection>> {
        listCollectionsCallCount++
        return collectionsResult
    }

    override suspend fun listSmartLists(): Result<List<SmartList>> {
        listSmartListsCallCount++
        return smartListsResult
    }

    companion object {
        private val fixedInstant = Instant.parse("2026-01-01T00:00:00Z")

        fun collection(
            id: String = "col_1",
            name: String = "Reading",
            color: String? = "blue",
            itemCount: Long = 3,
        ) = Collection(
            id = id,
            `object` = "collection",
            name = name,
            color = color,
            itemCount = itemCount,
            sortOrder = 0,
            createdAt = fixedInstant,
            updatedAt = fixedInstant,
        )

        fun smartList(
            id: String = "sl_1",
            name: String = "Unread",
            isPinned: Boolean = false,
        ) = SmartList(
            id = id,
            `object` = "smart_list",
            name = name,
            filterExpression = JsonObject(emptyMap()),
            isPinned = isPinned,
            createdAt = fixedInstant,
            updatedAt = fixedInstant,
        )
    }
}
