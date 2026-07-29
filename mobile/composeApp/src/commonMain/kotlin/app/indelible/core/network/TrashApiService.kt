package app.indelible.core.network

import app.indelible.api.generated.client.ApiV1LibraryPurgeClient
import app.indelible.api.generated.client.ApiV1LibraryRestoreClient
import app.indelible.api.generated.client.ApiV1LibraryTrashClient
import app.indelible.api.generated.client.ApiV1LibraryTrashEmptyClient
import app.indelible.api.generated.models.LibraryEntryResponse
import app.indelible.api.generated.models.PaginatedResponseLibraryEntryResponse

class TrashApiService(
    private val transport: AuthenticatedApiTransport,
) {
    suspend fun listTrash(
        cursor: String? = null,
        limit: Int = 50,
    ): Result<PaginatedResponseLibraryEntryResponse> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1LibraryTrashClient(client).listLibraryTrash(
                cursor = cursor,
                limit = limit,
                apiConfiguration = configuration,
            )
        }

    suspend fun emptyTrash(): Result<Unit> =
        transport
            .authenticatedRequest { client, configuration ->
                ApiV1LibraryTrashEmptyClient(client).emptyLibraryTrash(configuration)
            }.map { Unit }

    suspend fun restoreItem(itemId: String): Result<LibraryEntryResponse> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1LibraryRestoreClient(client).restoreEntry(itemId, configuration)
        }

    suspend fun permanentlyDeleteItem(itemId: String): Result<Unit> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1LibraryPurgeClient(client).purgeEntry(itemId, configuration)
        }
}
