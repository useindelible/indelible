package app.indelible.search.repository

import app.indelible.search.model.PaginatedSearchResults
import app.indelible.search.model.RecentSearch
import app.indelible.search.model.SearchSuggestion

interface SearchRepository {
    suspend fun search(
        query: String,
        cursor: String? = null,
        limit: Int = 20,
    ): Result<PaginatedSearchResults>

    suspend fun suggestions(
        query: String,
        limit: Int = 8,
    ): Result<List<SearchSuggestion>>

    suspend fun listRecentSearches(limit: Int = 20): Result<List<RecentSearch>>

    suspend fun deleteRecentSearch(id: String): Result<Unit>

    suspend fun clearRecentSearches(): Result<Unit>
}
