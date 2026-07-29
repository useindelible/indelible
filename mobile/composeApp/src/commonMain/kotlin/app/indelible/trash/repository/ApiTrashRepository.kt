package app.indelible.trash.repository

import app.indelible.core.model.ItemDetail
import app.indelible.core.model.PaginatedItems
import app.indelible.core.model.toLibraryItem
import app.indelible.core.model.toPaginatedItems
import app.indelible.core.network.TrashApiService

class ApiTrashRepository(
    private val trashApiService: TrashApiService,
) : TrashRepository {
    override suspend fun listTrash(cursor: String?): Result<PaginatedItems> =
        trashApiService.listTrash(cursor = cursor).map { it.toPaginatedItems() }

    override suspend fun restoreItem(itemId: String): Result<ItemDetail> = trashApiService.restoreItem(itemId).map { it.toLibraryItem() }

    override suspend fun emptyTrash(): Result<Unit> = trashApiService.emptyTrash()
}
