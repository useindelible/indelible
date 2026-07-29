package app.indelible.search.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import app.indelible.search.repository.SearchRepository
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

class SearchViewModel(
    private val repository: SearchRepository,
) : ViewModel() {
    private val _state = MutableStateFlow(SearchState())
    val state: StateFlow<SearchState> = _state.asStateFlow()

    private val _effects = MutableSharedFlow<SearchEffect>()
    val effects: SharedFlow<SearchEffect> = _effects.asSharedFlow()

    private var nextCursor: String? = null
    private var debounceJob: Job? = null
    private var suggestionsJob: Job? = null

    fun onQueryChange(query: String) {
        _state.update { it.copy(query = query) }
        debounceJob?.cancel()
        suggestionsJob?.cancel()
        if (query.isBlank()) {
            nextCursor = null
            _state.update {
                it.copy(
                    submittedQuery = "",
                    results = emptyList(),
                    hasMore = false,
                    error = null,
                    suggestions = emptyList(),
                    showSuggestions = false,
                )
            }
            loadRecentSearches()
            return
        }
        suggestionsJob =
            viewModelScope.launch {
                delay(SUGGESTIONS_DEBOUNCE_MS)
                fetchSuggestions(query)
            }
        debounceJob =
            viewModelScope.launch {
                delay(DEBOUNCE_MS)
                executeSearch(query)
            }
    }

    fun submitSearch(query: String = _state.value.query) {
        if (query.isBlank()) return
        debounceJob?.cancel()
        suggestionsJob?.cancel()
        _state.update { it.copy(query = query, showSuggestions = false, suggestions = emptyList()) }
        executeSearch(query)
    }

    fun selectRecentSearch(query: String) {
        debounceJob?.cancel()
        suggestionsJob?.cancel()
        submitSearch(query)
    }

    fun selectSuggestion(insertText: String) {
        val currentQuery = _state.value.query.trimEnd()
        val lastSpace = currentQuery.lastIndexOf(' ')
        val newQuery =
            if (lastSpace >= 0) {
                "${currentQuery.substring(0, lastSpace + 1)}$insertText"
            } else {
                insertText
            }
        debounceJob?.cancel()
        suggestionsJob?.cancel()
        _state.update {
            it.copy(query = newQuery, showSuggestions = false, suggestions = emptyList())
        }
        debounceJob =
            viewModelScope.launch {
                delay(DEBOUNCE_MS)
                executeSearch(newQuery)
            }
    }

    fun dismissSuggestions() {
        suggestionsJob?.cancel()
        _state.update { it.copy(showSuggestions = false) }
    }

    fun loadNextPage() {
        val s = _state.value
        if (!s.hasMore || s.isLoadingMore || s.submittedQuery.isBlank()) return
        _state.update { it.copy(isLoadingMore = true) }
        viewModelScope.launch {
            repository
                .search(query = s.submittedQuery, cursor = nextCursor)
                .onSuccess { paginated ->
                    nextCursor = paginated.nextCursor
                    _state.update {
                        it.copy(
                            results = it.results + paginated.results,
                            hasMore = paginated.hasMore,
                            isLoadingMore = false,
                        )
                    }
                }.onFailure { error ->
                    _state.update { it.copy(isLoadingMore = false) }
                    _effects.emit(SearchEffect.ShowSnackbar(error.message ?: "Failed to load more results"))
                }
        }
    }

    fun refresh() {
        val s = _state.value
        if (s.submittedQuery.isBlank()) {
            loadRecentSearches()
            return
        }
        _state.update { it.copy(isRefreshing = true) }
        nextCursor = null
        viewModelScope.launch {
            repository
                .search(query = s.submittedQuery, cursor = null)
                .onSuccess { paginated ->
                    nextCursor = paginated.nextCursor
                    _state.update {
                        it.copy(
                            results = paginated.results,
                            hasMore = paginated.hasMore,
                            isRefreshing = false,
                        )
                    }
                }.onFailure { error ->
                    _state.update { it.copy(isRefreshing = false) }
                    _effects.emit(SearchEffect.ShowSnackbar(error.message ?: "Refresh failed"))
                }
        }
    }

    fun clearQuery() {
        debounceJob?.cancel()
        suggestionsJob?.cancel()
        nextCursor = null
        _state.update {
            it.copy(
                query = "",
                submittedQuery = "",
                results = emptyList(),
                hasMore = false,
                error = null,
                suggestions = emptyList(),
                showSuggestions = false,
            )
        }
        loadRecentSearches()
    }

    fun onResultTap(itemId: String) {
        viewModelScope.launch {
            _effects.emit(SearchEffect.NavigateToReader(itemId))
        }
    }

    fun deleteRecentSearch(id: String) {
        _state.update { s -> s.copy(recentSearches = s.recentSearches.filter { it.id != id }) }
        viewModelScope.launch {
            repository.deleteRecentSearch(id).onFailure {
                loadRecentSearches()
            }
        }
    }

    fun clearRecentSearches() {
        _state.update { it.copy(recentSearches = emptyList()) }
        viewModelScope.launch {
            repository.clearRecentSearches().onFailure {
                loadRecentSearches()
            }
        }
    }

    private fun loadRecentSearches() {
        _state.update { it.copy(isLoadingRecent = true) }
        viewModelScope.launch {
            repository
                .listRecentSearches()
                .onSuccess { recent ->
                    _state.update { it.copy(recentSearches = recent, isLoadingRecent = false) }
                }.onFailure {
                    _state.update { it.copy(isLoadingRecent = false) }
                }
        }
    }

    private fun executeSearch(query: String) {
        nextCursor = null
        _state.update {
            it.copy(submittedQuery = query, isSearching = true, results = emptyList(), error = null)
        }
        viewModelScope.launch {
            repository
                .search(query = query, cursor = null)
                .onSuccess { paginated ->
                    nextCursor = paginated.nextCursor
                    _state.update {
                        it.copy(
                            results = paginated.results,
                            hasMore = paginated.hasMore,
                            isSearching = false,
                        )
                    }
                }.onFailure { error ->
                    _state.update { it.copy(isSearching = false, error = error.message ?: "Search failed") }
                }
        }
    }

    private suspend fun fetchSuggestions(query: String) {
        repository
            .suggestions(query)
            .onSuccess { suggestions ->
                _state.update {
                    it.copy(suggestions = suggestions, showSuggestions = suggestions.isNotEmpty())
                }
            }
        // Suggestion failures are silent — they don't block the search experience
    }

    companion object {
        private const val DEBOUNCE_MS = 400L
        private const val SUGGESTIONS_DEBOUNCE_MS = 200L
    }
}
