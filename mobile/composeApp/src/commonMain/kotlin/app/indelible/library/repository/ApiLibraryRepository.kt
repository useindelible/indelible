package app.indelible.library.repository

import app.indelible.api.generated.models.PaginatedResponseLibraryEntryResponse
import app.indelible.core.model.ItemDetail
import app.indelible.core.model.LibraryCounts
import app.indelible.core.model.PaginatedItems
import app.indelible.core.model.toLibraryCounts
import app.indelible.core.model.toLibraryItem
import app.indelible.core.model.toPaginatedItems
import app.indelible.core.network.LibraryApiService
import app.indelible.core.offline.ListSnapshots
import app.indelible.core.offline.orFromDevice

class ApiLibraryRepository(
    private val libraryApiService: LibraryApiService,
    private val snapshots: ListSnapshots,
) : LibraryRepository {
    override suspend fun listItems(
        triageState: String?,
        itemType: String?,
        cursor: String?,
        limit: Int,
    ): Result<PaginatedItems> =
        page(triageKey(triageState, itemType), cursor) {
            libraryApiService.listItems(triageState, itemType, cursor, limit)
        }

    override suspend fun listCollectionItems(
        collectionId: String,
        cursor: String?,
        limit: Int,
    ): Result<PaginatedItems> =
        page(collectionKey(collectionId), cursor) {
            libraryApiService.listCollectionItems(collectionId, cursor, limit)
        }

    override suspend fun listSmartListItems(
        smartListId: String,
        cursor: String?,
        limit: Int,
    ): Result<PaginatedItems> =
        page(smartListKey(smartListId), cursor) {
            libraryApiService.listSmartListItems(smartListId, cursor, limit)
        }

    override suspend fun scopeCounts(triageState: String?): Result<LibraryCounts> =
        libraryApiService.getScopeCounts(triageState).map { it.toLibraryCounts() }

    override suspend fun getItem(itemId: String): Result<ItemDetail> =
        libraryApiService.getItem(itemId).map { it.toLibraryItem() }

    override suspend fun triageItem(
        itemId: String,
        state: String,
    ): Result<ItemDetail> {
        val session = snapshots.currentSession()
        return libraryApiService
            .triageItem(itemId, state)
            .onSuccess { snapshots.forget(session, itemId, keepAfterTriage(it.triageState)) }
            .map { it.toLibraryItem() }
    }

    override suspend fun toggleFavorite(itemId: String): Result<ItemDetail> =
        libraryApiService.toggleFavorite(itemId).map { it.toLibraryItem() }

    override suspend fun toggleShortlist(itemId: String): Result<ItemDetail> =
        libraryApiService.toggleShortlist(itemId).map { it.toLibraryItem() }

    override suspend fun deleteItem(itemId: String): Result<Unit> {
        val session = snapshots.currentSession()
        return libraryApiService.deleteItem(itemId).onSuccess { snapshots.forget(session, itemId) }
    }

    override suspend fun rearchiveItem(itemId: String): Result<ItemDetail> {
        val session = snapshots.currentSession()
        return libraryApiService
            .rearchiveItem(itemId)
            .onSuccess { snapshots.forget(session, itemId, keepAfterTriage(it.triageState)) }
            .map { it.toLibraryItem() }
    }

    private suspend fun page(
        key: String,
        cursor: String?,
        fetch: suspend () -> Result<PaginatedResponseLibraryEntryResponse>,
    ): Result<PaginatedItems> {
        val session = snapshots.currentSession()
        val result = fetch()
        result.onSuccess { response ->
            snapshots.keep(session, key, response.data.map { it.toSnapshotRow() }, append = cursor != null)
        }
        val page = result.map { it.toPaginatedItems() }
        if (cursor != null) return page
        return page.orFromDevice { snapshots.kept(session, key).toDevicePage() }
    }
}
