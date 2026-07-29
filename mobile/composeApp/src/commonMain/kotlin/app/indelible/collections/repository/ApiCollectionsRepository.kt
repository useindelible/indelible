package app.indelible.collections.repository

import app.indelible.api.generated.models.CollectionResponse
import app.indelible.api.generated.models.PaginatedResponseCollectionResponse
import app.indelible.core.model.PaginatedItems
import app.indelible.core.model.toPaginatedItems
import app.indelible.core.network.CollectionsApiService

class ApiCollectionsRepository(
    private val collectionsApiService: CollectionsApiService,
) : CollectionsRepository {
    override suspend fun listCollections(
        cursor: String?,
        limit: Int,
    ): Result<PaginatedResponseCollectionResponse> = collectionsApiService.listCollections(cursor, limit)

    override suspend fun getCollection(id: String): Result<CollectionResponse> = collectionsApiService.getCollection(id)

    override suspend fun listCollectionChildren(
        id: String,
        cursor: String?,
        limit: Int,
    ): Result<PaginatedResponseCollectionResponse> = collectionsApiService.listCollectionChildren(id, cursor, limit)

    override suspend fun listCollectionItems(
        id: String,
        cursor: String?,
        limit: Int,
    ): Result<PaginatedItems> = collectionsApiService.listCollectionItems(id, cursor, limit).map { it.toPaginatedItems() }
}
