package app.indelible.library.viewmodel

import app.indelible.core.model.ItemDetail
import app.indelible.core.model.LibraryCounts
import app.indelible.core.model.LibraryItem
import app.indelible.core.model.PageInfo
import app.indelible.core.model.PaginatedItems
import app.indelible.library.repository.LibraryRepository
import kotlinx.datetime.Instant

class FakeLibraryRepository : LibraryRepository {
    var listItemsResult: Result<PaginatedItems> = Result.success(emptyPaginatedItems())
    var listCollectionItemsResult: Result<PaginatedItems> = Result.success(emptyPaginatedItems())
    var listSmartListItemsResult: Result<PaginatedItems> = Result.success(emptyPaginatedItems())
    var getItemResult: Result<ItemDetail> = Result.success(fakeItemDetail())
    var triageItemResult: Result<ItemDetail> = Result.success(fakeItemDetail())
    var toggleFavoriteResult: Result<ItemDetail> = Result.success(fakeItemDetail())
    var toggleShortlistResult: Result<ItemDetail> = Result.success(fakeItemDetail())
    var deleteItemResult: Result<Unit> = Result.success(Unit)
    var rearchiveItemResult: Result<ItemDetail> = Result.success(fakeItemDetail())
    var scopeCountsResult: Result<LibraryCounts> = Result.success(LibraryCounts.EMPTY)

    var lastTriagedItemId: String? = null
    var lastTriagedState: String? = null
    var listItemsCallCount = 0
    var lastListItemsTriageState: String? = null
    var lastListItemsItemType: String? = null
    var lastListItemsCursor: String? = null
    var lastCollectionItemsId: String? = null
    var lastCollectionItemsCursor: String? = null
    var lastSmartListItemsId: String? = null
    var lastSmartListItemsCursor: String? = null
    var scopeCountsCallCount = 0
    var lastScopeCountsTriageState: String? = null

    override suspend fun listItems(
        triageState: String?,
        itemType: String?,
        cursor: String?,
        limit: Int,
    ): Result<PaginatedItems> {
        listItemsCallCount++
        lastListItemsTriageState = triageState
        lastListItemsItemType = itemType
        lastListItemsCursor = cursor
        return listItemsResult
    }

    override suspend fun listCollectionItems(
        collectionId: String,
        cursor: String?,
        limit: Int,
    ): Result<PaginatedItems> {
        lastCollectionItemsId = collectionId
        lastCollectionItemsCursor = cursor
        return listCollectionItemsResult
    }

    override suspend fun listSmartListItems(
        smartListId: String,
        cursor: String?,
        limit: Int,
    ): Result<PaginatedItems> {
        lastSmartListItemsId = smartListId
        lastSmartListItemsCursor = cursor
        return listSmartListItemsResult
    }

    override suspend fun scopeCounts(triageState: String?): Result<LibraryCounts> {
        scopeCountsCallCount++
        lastScopeCountsTriageState = triageState
        return scopeCountsResult
    }

    override suspend fun getItem(itemId: String): Result<ItemDetail> = getItemResult

    override suspend fun triageItem(
        itemId: String,
        state: String,
    ): Result<ItemDetail> {
        lastTriagedItemId = itemId
        lastTriagedState = state
        return triageItemResult
    }

    override suspend fun toggleFavorite(itemId: String): Result<ItemDetail> = toggleFavoriteResult

    override suspend fun toggleShortlist(itemId: String): Result<ItemDetail> = toggleShortlistResult

    override suspend fun deleteItem(itemId: String): Result<Unit> = deleteItemResult

    override suspend fun rearchiveItem(itemId: String): Result<ItemDetail> = rearchiveItemResult

    companion object {
        fun emptyPaginatedItems() =
            PaginatedItems(
                data = emptyList(),
                page = PageInfo(nextCursor = null, hasMore = false),
            )

        fun paginatedItems(
            items: List<LibraryItem>,
            hasMore: Boolean = false,
            nextCursor: String? = null,
        ) = PaginatedItems(
            data = items,
            page = PageInfo(nextCursor = nextCursor, hasMore = hasMore),
        )

        fun fakeLibraryItem(
            id: String = "item1",
            triageState: String = "inbox",
        ) = LibraryItem(
            id = id,
            documentId = "doc_$id",
            itemType = "article",
            triageState = triageState,
            isFavorite = false,
            isShortlisted = false,
            title = "Test Article $id",
            excerpt = "Test excerpt",
            domain = "example.com",
            source = "url",
            savedAt = Instant.parse("2024-01-01T00:00:00Z"),
            createdAt = Instant.parse("2024-01-01T00:00:00Z"),
            updatedAt = Instant.parse("2024-01-01T00:00:00Z"),
        )

        fun fakeItemDetail(
            id: String = "item1",
            isFavorite: Boolean = false,
        ) = ItemDetail(
            id = id,
            documentId = "doc_$id",
            itemType = "article",
            triageState = "inbox",
            isFavorite = isFavorite,
            isShortlisted = false,
            title = "Test Article",
            source = "url",
            savedAt = Instant.parse("2024-01-01T00:00:00Z"),
            createdAt = Instant.parse("2024-01-01T00:00:00Z"),
            updatedAt = Instant.parse("2024-01-01T00:00:00Z"),
        )
    }
}
