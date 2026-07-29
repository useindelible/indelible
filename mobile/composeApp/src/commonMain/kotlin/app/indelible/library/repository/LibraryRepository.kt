package app.indelible.library.repository

import app.indelible.core.model.ItemDetail
import app.indelible.core.model.LibraryCounts
import app.indelible.core.model.PaginatedItems

interface LibraryRepository {
    suspend fun listItems(
        triageState: String? = null,
        itemType: String? = null,
        cursor: String? = null,
        limit: Int = 50,
    ): Result<PaginatedItems>

    suspend fun listCollectionItems(
        collectionId: String,
        cursor: String? = null,
        limit: Int = 50,
    ): Result<PaginatedItems>

    suspend fun listSmartListItems(
        smartListId: String,
        cursor: String? = null,
        limit: Int = 50,
    ): Result<PaginatedItems>

    /** Read-state and item-type totals for a triage scope, or the whole library when null. */
    suspend fun scopeCounts(triageState: String? = null): Result<LibraryCounts>

    suspend fun getItem(itemId: String): Result<ItemDetail>

    suspend fun triageItem(
        itemId: String,
        state: String,
    ): Result<ItemDetail>

    suspend fun toggleFavorite(itemId: String): Result<ItemDetail>

    suspend fun toggleShortlist(itemId: String): Result<ItemDetail>

    suspend fun deleteItem(itemId: String): Result<Unit>

    suspend fun rearchiveItem(itemId: String): Result<ItemDetail>
}
