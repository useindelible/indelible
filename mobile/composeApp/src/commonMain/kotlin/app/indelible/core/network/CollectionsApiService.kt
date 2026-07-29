package app.indelible.core.network

import app.indelible.api.generated.client.ApiV1CollectionsChildrenClient
import app.indelible.api.generated.client.ApiV1CollectionsClient
import app.indelible.api.generated.client.ApiV1CollectionsEntriesClient
import app.indelible.api.generated.client.ApiV1SmartListsClient
import app.indelible.api.generated.client.ApiV1SmartListsEntriesClient
import app.indelible.api.generated.client.ApiV1SmartListsPinClient
import app.indelible.api.generated.models.AddLibraryEntryBody
import app.indelible.api.generated.models.CollectionResponse
import app.indelible.api.generated.models.CreateCollectionBody
import app.indelible.api.generated.models.CreateSmartListBody
import app.indelible.api.generated.models.PaginatedResponseCollectionResponse
import app.indelible.api.generated.models.PaginatedResponseLibraryEntryResponse
import app.indelible.api.generated.models.PaginatedResponseSmartListResponse
import app.indelible.api.generated.models.PinSmartListBody
import app.indelible.api.generated.models.SmartListResponse
import app.indelible.api.generated.models.UpdateCollectionBody
import app.indelible.api.generated.models.UpdateSmartListBody

class CollectionsApiService(
    private val transport: AuthenticatedApiTransport,
) {
    suspend fun listCollections(
        cursor: String? = null,
        limit: Int = 50,
    ): Result<PaginatedResponseCollectionResponse> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1CollectionsClient(client).listCollections(cursor, limit, configuration)
        }

    suspend fun createCollection(body: CreateCollectionBody): Result<CollectionResponse> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1CollectionsClient(client).createCollection(body, configuration)
        }

    suspend fun getCollection(id: String): Result<CollectionResponse> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1CollectionsClient(client).getCollection(id, configuration)
        }

    suspend fun updateCollection(
        id: String,
        body: UpdateCollectionBody,
    ): Result<CollectionResponse> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1CollectionsClient(client).updateCollection(body, id, configuration)
        }

    suspend fun deleteCollection(id: String): Result<Unit> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1CollectionsClient(client).deleteCollection(id, configuration)
        }

    suspend fun listCollectionChildren(
        id: String,
        cursor: String? = null,
        limit: Int = 50,
    ): Result<PaginatedResponseCollectionResponse> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1CollectionsChildrenClient(client).listChildren(id, cursor, limit, configuration)
        }

    suspend fun listCollectionItems(
        id: String,
        cursor: String? = null,
        limit: Int = 50,
    ): Result<PaginatedResponseLibraryEntryResponse> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1CollectionsEntriesClient(client).listCollectionEntries(id, cursor, limit, configuration)
        }

    suspend fun addItemToCollection(
        collectionId: String,
        itemId: String,
    ): Result<Unit> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1CollectionsEntriesClient(client).addEntryToCollection(
                AddLibraryEntryBody(libraryEntryId = itemId),
                collectionId,
                configuration,
            )
        }

    suspend fun removeItemFromCollection(
        collectionId: String,
        itemId: String,
    ): Result<Unit> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1CollectionsEntriesClient(client).removeEntryFromCollection(
                collectionId,
                itemId,
                configuration,
            )
        }

    suspend fun listSmartLists(
        cursor: String? = null,
        limit: Int = 50,
    ): Result<PaginatedResponseSmartListResponse> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1SmartListsClient(client).listSmartLists(cursor, limit, configuration)
        }

    suspend fun createSmartList(body: CreateSmartListBody): Result<SmartListResponse> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1SmartListsClient(client).createSmartList(body, configuration)
        }

    suspend fun getSmartList(id: String): Result<SmartListResponse> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1SmartListsClient(client).getSmartList(id, configuration)
        }

    suspend fun updateSmartList(
        id: String,
        body: UpdateSmartListBody,
    ): Result<SmartListResponse> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1SmartListsClient(client).updateSmartList(body, id, configuration)
        }

    suspend fun deleteSmartList(id: String): Result<Unit> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1SmartListsClient(client).deleteSmartList(id, configuration)
        }

    suspend fun listSmartListItems(
        id: String,
        cursor: String? = null,
        limit: Int = 50,
    ): Result<PaginatedResponseLibraryEntryResponse> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1SmartListsEntriesClient(client).evaluateSmartListEntries(id, cursor, limit, configuration)
        }

    suspend fun pinSmartList(
        id: String,
        isPinned: Boolean,
    ): Result<SmartListResponse> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1SmartListsPinClient(client).pinSmartList(PinSmartListBody(isPinned), id, configuration)
        }
}
