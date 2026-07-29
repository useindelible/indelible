package app.indelible.search.repository

import app.indelible.core.network.SearchApiService
import app.indelible.search.model.PaginatedSearchResults
import app.indelible.search.model.RecentSearch
import app.indelible.search.model.SearchSuggestion

class ApiSearchRepository(
    private val searchApiService: SearchApiService,
) : SearchRepository {
    override suspend fun search(
        query: String,
        cursor: String?,
        limit: Int,
    ): Result<PaginatedSearchResults> = searchApiService.search(query, cursor, limit)

    override suspend fun suggestions(
        query: String,
        limit: Int,
    ): Result<List<SearchSuggestion>> = searchApiService.suggestions(query, limit)

    override suspend fun listRecentSearches(limit: Int): Result<List<RecentSearch>> = searchApiService.listRecentSearches(limit)

    override suspend fun deleteRecentSearch(id: String): Result<Unit> = searchApiService.deleteRecentSearch(id)

    override suspend fun clearRecentSearches(): Result<Unit> = searchApiService.clearRecentSearches()
}
