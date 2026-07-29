package app.indelible.trash.repository

import app.indelible.core.model.ItemDetail
import app.indelible.core.model.PaginatedItems

interface TrashRepository {
    suspend fun listTrash(cursor: String? = null): Result<PaginatedItems>

    suspend fun restoreItem(itemId: String): Result<ItemDetail>

    suspend fun emptyTrash(): Result<Unit>
}
