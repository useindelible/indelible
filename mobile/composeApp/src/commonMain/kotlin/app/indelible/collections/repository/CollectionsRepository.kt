package app.indelible.collections.repository

import app.indelible.api.generated.models.CollectionResponse
import app.indelible.api.generated.models.PaginatedResponseCollectionResponse
import app.indelible.core.model.PaginatedItems

interface CollectionsRepository {
    suspend fun listCollections(
        cursor: String? = null,
        limit: Int = 50,
    ): Result<PaginatedResponseCollectionResponse>

    suspend fun getCollection(id: String): Result<CollectionResponse>

    suspend fun listCollectionChildren(
        id: String,
        cursor: String? = null,
        limit: Int = 50,
    ): Result<PaginatedResponseCollectionResponse>

    suspend fun listCollectionItems(
        id: String,
        cursor: String? = null,
        limit: Int = 50,
    ): Result<PaginatedItems>
}
