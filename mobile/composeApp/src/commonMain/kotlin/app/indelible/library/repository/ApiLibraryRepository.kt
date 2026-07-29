package app.indelible.library.repository

import app.indelible.core.model.ItemDetail
import app.indelible.core.model.LibraryCounts
import app.indelible.core.model.PaginatedItems
import app.indelible.core.model.toLibraryCounts
import app.indelible.core.model.toLibraryItem
import app.indelible.core.model.toPaginatedItems
import app.indelible.core.network.LibraryApiService

class ApiLibraryRepository(
    private val libraryApiService: LibraryApiService,
) : LibraryRepository {
    override suspend fun listItems(
        triageState: String?,
        itemType: String?,
        cursor: String?,
        limit: Int,
    ): Result<PaginatedItems> =
        libraryApiService.listItems(triageState, itemType, cursor, limit).map { it.toPaginatedItems() }

    override suspend fun listCollectionItems(
        collectionId: String,
        cursor: String?,
        limit: Int,
    ): Result<PaginatedItems> =
        libraryApiService.listCollectionItems(collectionId, cursor, limit).map { it.toPaginatedItems() }

    override suspend fun listSmartListItems(
        smartListId: String,
        cursor: String?,
        limit: Int,
    ): Result<PaginatedItems> =
        libraryApiService.listSmartListItems(smartListId, cursor, limit).map { it.toPaginatedItems() }

    override suspend fun scopeCounts(triageState: String?): Result<LibraryCounts> =
        libraryApiService.getScopeCounts(triageState).map { it.toLibraryCounts() }

    override suspend fun getItem(itemId: String): Result<ItemDetail> =
        libraryApiService.getItem(itemId).map { it.toLibraryItem() }

    override suspend fun triageItem(
        itemId: String,
        state: String,
    ): Result<ItemDetail> = libraryApiService.triageItem(itemId, state).map { it.toLibraryItem() }

    override suspend fun toggleFavorite(itemId: String): Result<ItemDetail> =
        libraryApiService.toggleFavorite(itemId).map { it.toLibraryItem() }

    override suspend fun toggleShortlist(itemId: String): Result<ItemDetail> =
        libraryApiService.toggleShortlist(itemId).map { it.toLibraryItem() }

    override suspend fun deleteItem(itemId: String): Result<Unit> = libraryApiService.deleteItem(itemId)

    override suspend fun rearchiveItem(itemId: String): Result<ItemDetail> =
        libraryApiService.rearchiveItem(itemId).map { it.toLibraryItem() }
}
