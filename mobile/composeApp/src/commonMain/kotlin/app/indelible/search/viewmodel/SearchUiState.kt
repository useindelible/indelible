package app.indelible.search.viewmodel

import app.indelible.search.model.RecentSearch
import app.indelible.search.model.SearchResult
import app.indelible.search.model.SearchSuggestion

data class SearchState(
    val query: String = "",
    val submittedQuery: String = "",
    val results: List<SearchResult> = emptyList(),
    val recentSearches: List<RecentSearch> = emptyList(),
    val suggestions: List<SearchSuggestion> = emptyList(),
    val showSuggestions: Boolean = false,
    val isSearching: Boolean = false,
    val isLoadingMore: Boolean = false,
    val isRefreshing: Boolean = false,
    val isLoadingRecent: Boolean = false,
    val hasMore: Boolean = false,
    val error: String? = null,
)

sealed class SearchEffect {
    data class NavigateToReader(
        val itemId: String,
    ) : SearchEffect()

    data class ShowSnackbar(
        val message: String,
    ) : SearchEffect()
}
