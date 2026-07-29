package app.indelible.core.network

import app.indelible.api.generated.client.ApiV1CollectionsEntriesClient
import app.indelible.api.generated.client.ApiV1LibraryClient
import app.indelible.api.generated.client.ApiV1LibraryCountClient
import app.indelible.api.generated.client.ApiV1LibraryCountsClient
import app.indelible.api.generated.client.ApiV1LibraryFavoriteClient
import app.indelible.api.generated.client.ApiV1LibraryQueryClient
import app.indelible.api.generated.client.ApiV1LibraryRestoreClient
import app.indelible.api.generated.client.ApiV1LibraryShortlistClient
import app.indelible.api.generated.client.ApiV1LibraryTriageClient
import app.indelible.api.generated.client.ApiV1SmartListsEntriesClient
import app.indelible.api.generated.models.LibraryCountResponse
import app.indelible.api.generated.models.LibraryEntryResponse
import app.indelible.api.generated.models.LibraryQueryBody
import app.indelible.api.generated.models.LibraryScopeCountsResponse
import app.indelible.api.generated.models.LibraryTriageBody
import app.indelible.api.generated.models.PaginatedResponseLibraryEntryResponse
import app.indelible.core.model.SaveItemRequest
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.addJsonObject
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.put
import kotlinx.serialization.json.putJsonArray

class LibraryApiService(
    private val transport: AuthenticatedApiTransport,
) {
    suspend fun saveItem(request: SaveItemRequest): Result<LibraryEntryResponse> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1LibraryClient(client).saveUrl(request, configuration)
        }

    suspend fun getLibraryCount(): Result<LibraryCountResponse> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1LibraryCountClient(client).countLibrary(configuration)
        }

    suspend fun getScopeCounts(triageState: String? = null): Result<LibraryScopeCountsResponse> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1LibraryCountsClient(client).libraryCounts(triageState, configuration)
        }

    suspend fun listItems(
        triageState: String? = null,
        itemType: String? = null,
        cursor: String? = null,
        limit: Int = 50,
    ): Result<PaginatedResponseLibraryEntryResponse> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1LibraryQueryClient(client).queryLibrary(
                libraryQueryBody =
                    LibraryQueryBody(
                        filterExpression = libraryFilterExpression(triageState, itemType),
                        cursor = cursor,
                        limit = limit,
                    ),
                apiConfiguration = configuration,
            )
        }

    suspend fun listCollectionItems(
        id: String,
        cursor: String? = null,
        limit: Int = 50,
    ): Result<PaginatedResponseLibraryEntryResponse> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1CollectionsEntriesClient(client).listCollectionEntries(
                id = id,
                cursor = cursor,
                limit = limit,
                apiConfiguration = configuration,
            )
        }

    suspend fun listSmartListItems(
        id: String,
        cursor: String? = null,
        limit: Int = 50,
    ): Result<PaginatedResponseLibraryEntryResponse> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1SmartListsEntriesClient(client).evaluateSmartListEntries(
                id = id,
                cursor = cursor,
                limit = limit,
                apiConfiguration = configuration,
            )
        }

    suspend fun getItem(itemId: String): Result<LibraryEntryResponse> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1LibraryClient(client).getLibraryEntry(itemId, configuration)
        }

    suspend fun triageItem(
        itemId: String,
        state: String,
    ): Result<LibraryEntryResponse> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1LibraryTriageClient(client).triageEntry(
                libraryTriageBody = LibraryTriageBody(triageState = state),
                libraryEntryId = itemId,
                apiConfiguration = configuration,
            )
        }

    suspend fun toggleFavorite(itemId: String): Result<LibraryEntryResponse> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1LibraryFavoriteClient(client).toggleLibraryFavorite(itemId, configuration)
        }

    suspend fun toggleShortlist(itemId: String): Result<LibraryEntryResponse> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1LibraryShortlistClient(client).toggleLibraryShortlist(itemId, configuration)
        }

    suspend fun deleteItem(itemId: String): Result<Unit> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1LibraryClient(client).deleteLibraryEntry(itemId, configuration)
        }

    suspend fun rearchiveItem(itemId: String): Result<LibraryEntryResponse> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1LibraryRestoreClient(client).restoreEntry(itemId, configuration)
        }

    private fun libraryFilterExpression(
        triageState: String?,
        itemType: String?,
    ): JsonElement? {
        val conditions =
            buildList {
                if (itemType != null) add("item_type" to itemType)
                if (triageState != null) add("triage_state" to triageState)
            }
        if (conditions.isEmpty()) return null
        return buildJsonObject {
            put("type", "and")
            putJsonArray("conditions") {
                conditions.forEach { (field, value) ->
                    addJsonObject {
                        put("type", "condition")
                        put("field", field)
                        put("op", "eq")
                        put("value", value)
                    }
                }
            }
        }
    }
}
