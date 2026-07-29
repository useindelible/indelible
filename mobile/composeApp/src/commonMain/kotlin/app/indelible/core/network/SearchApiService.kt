package app.indelible.core.network

import app.indelible.api.generated.client.ApiV1SearchClient
import app.indelible.api.generated.client.ApiV1SearchRecentClient
import app.indelible.api.generated.client.ApiV1SearchSuggestionsClient
import app.indelible.search.model.PaginatedSearchResults
import app.indelible.search.model.RecentSearch
import app.indelible.search.model.SearchSuggestion

class SearchApiService(
    private val transport: AuthenticatedApiTransport,
) {
    suspend fun search(
        query: String,
        cursor: String? = null,
        limit: Int = 20,
    ): Result<PaginatedSearchResults> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1SearchClient(client).search(query, cursor, limit, configuration)
        }

    suspend fun suggestions(
        query: String,
        limit: Int = 8,
    ): Result<List<SearchSuggestion>> =
        transport
            .authenticatedRequest { client, configuration ->
                ApiV1SearchSuggestionsClient(client).suggestions(query, limit, configuration)
            }.map { it.suggestions }

    suspend fun listRecentSearches(limit: Int = 20): Result<List<RecentSearch>> =
        transport
            .authenticatedRequest { client, configuration ->
                ApiV1SearchRecentClient(client).listRecentSearches(limit, configuration)
            }.map { it.items }

    suspend fun deleteRecentSearch(id: String): Result<Unit> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1SearchRecentClient(client).deleteRecentSearch(id, configuration)
        }

    suspend fun clearRecentSearches(): Result<Unit> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1SearchRecentClient(client).clearRecentSearches(configuration)
        }
}
