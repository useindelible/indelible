package app.indelible.core.network

import app.indelible.api.generated.client.ApiV1HighlightsRecentClient
import app.indelible.api.generated.client.ApiV1HighlightsTagsClient
import app.indelible.api.generated.client.ApiV1TagsClient
import app.indelible.api.generated.client.ApiV1TagsEntriesClient
import app.indelible.api.generated.client.ApiV1TagsHighlightsClient
import app.indelible.api.generated.client.ApiV1TagsMergeClient
import app.indelible.api.generated.models.CreateTagBody
import app.indelible.api.generated.models.MergeTagsBody
import app.indelible.api.generated.models.PaginatedResponseHighlightResponse
import app.indelible.api.generated.models.PaginatedResponseLibraryEntryResponse
import app.indelible.api.generated.models.RecentHighlightsResponse
import app.indelible.api.generated.models.TagResponse
import app.indelible.api.generated.models.UpdateTagBody

class TagsApiService(
    private val transport: AuthenticatedApiTransport,
) {
    suspend fun listTags(
        scope: String? = null,
        limit: Int = 100,
    ): Result<List<TagResponse>> =
        transport
            .authenticatedRequest { client, configuration ->
                ApiV1TagsClient(client).listTags(limit = limit, scope = scope, apiConfiguration = configuration)
            }.map { it.data }

    suspend fun createTag(body: CreateTagBody): Result<TagResponse> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1TagsClient(client).createTag(body, configuration)
        }

    suspend fun mergeTags(body: MergeTagsBody): Result<TagResponse> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1TagsMergeClient(client).mergeTags(body, configuration)
        }

    suspend fun getTag(id: String): Result<TagResponse> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1TagsClient(client).getTag(id, configuration)
        }

    suspend fun updateTag(
        id: String,
        body: UpdateTagBody,
    ): Result<TagResponse> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1TagsClient(client).updateTag(body, id, configuration)
        }

    suspend fun deleteTag(id: String): Result<Unit> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1TagsClient(client).deleteTag(id, configuration)
        }

    suspend fun listTagHighlights(
        id: String,
        cursor: String? = null,
        limit: Int = 20,
    ): Result<PaginatedResponseHighlightResponse> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1TagsHighlightsClient(client).listTagHighlights(
                id = id,
                cursor = cursor,
                limit = limit,
                apiConfiguration = configuration,
            )
        }

    suspend fun listTagItems(
        id: String,
        cursor: String? = null,
        limit: Int = 50,
    ): Result<PaginatedResponseLibraryEntryResponse> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1TagsEntriesClient(client).listTagEntries(
                id = id,
                cursor = cursor,
                limit = limit,
                apiConfiguration = configuration,
            )
        }

    suspend fun listRecentHighlights(limit: Int = 20): Result<RecentHighlightsResponse> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1HighlightsRecentClient(client).listRecentHighlights(limit.toLong(), configuration)
        }

    suspend fun getHighlightTags(highlightId: String): Result<List<String>> =
        transport
            .authenticatedRequest { client, configuration ->
                ApiV1HighlightsTagsClient(client).getHighlightTags(highlightId, configuration)
            }.map { it.tags }
}
