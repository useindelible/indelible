package app.indelible.library.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import app.indelible.core.i18n.UiMessage
import app.indelible.core.model.LibraryCounts
import app.indelible.core.model.LibraryItem
import app.indelible.library.repository.LibraryRepository
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.library_error_delete
import indelible.composeapp.generated.resources.library_error_load
import indelible.composeapp.generated.resources.library_error_load_more
import indelible.composeapp.generated.resources.library_error_triage
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

class LibraryViewModel(
    private val repository: LibraryRepository,
) : ViewModel() {
    private val _uiState = MutableStateFlow<LibraryUiState>(LibraryUiState.Loading)
    val uiState: StateFlow<LibraryUiState> = _uiState.asStateFlow()

    private val _effects = MutableSharedFlow<LibraryEffect>()
    val effects: SharedFlow<LibraryEffect> = _effects.asSharedFlow()

    private val _triageFilter = MutableStateFlow(TriageFilter.INBOX)
    val triageFilter: StateFlow<TriageFilter> = _triageFilter.asStateFlow()

    private val _contentTypeFilter = MutableStateFlow(ContentTypeFilter.ALL)
    val contentTypeFilter: StateFlow<ContentTypeFilter> = _contentTypeFilter.asStateFlow()

    private val _scope = MutableStateFlow<LibraryScope>(LibraryScope.Triage)
    val scope: StateFlow<LibraryScope> = _scope.asStateFlow()

    /**
     * Header meter/chip totals for the active triage scope. Null while unknown and for
     * collection/smart-list scopes, which the counts endpoint does not describe.
     */
    private val _counts = MutableStateFlow<LibraryCounts?>(null)
    val counts: StateFlow<LibraryCounts?> = _counts.asStateFlow()

    private var nextCursor: String? = null

    fun setScope(scope: LibraryScope) {
        _scope.value = scope
        nextCursor = null
        loadItems()
        loadCounts()
    }

    fun setTriageFilter(filter: TriageFilter) {
        _triageFilter.value = filter
        _scope.value = LibraryScope.Triage
        nextCursor = null
        loadItems()
        loadCounts()
    }

    fun setContentTypeFilter(filter: ContentTypeFilter) {
        _contentTypeFilter.value = filter
        _scope.value = LibraryScope.Triage
        nextCursor = null
        loadItems()
        loadCounts()
    }

    fun refresh() {
        val current = _uiState.value
        if (current is LibraryUiState.Success) {
            _uiState.value = current.copy(isRefreshing = true)
        }
        nextCursor = null
        loadItems(isRefresh = true)
        loadCounts()
    }

    fun loadNextPage() {
        val current = _uiState.value as? LibraryUiState.Success ?: return
        if (!current.hasMore || current.isLoadingMore) return
        _uiState.value = current.copy(isLoadingMore = true)
        loadItems(append = true)
    }

    fun triageItem(
        item: LibraryItem,
        state: String,
    ) {
        val current = _uiState.value as? LibraryUiState.Success ?: return
        val updatedItems = current.items.toMutableList()
        val removedIndex = updatedItems.indexOfFirst { it.id == item.id }
        if (removedIndex == -1) return

        updatedItems.removeAt(removedIndex)
        _uiState.value = current.copy(items = updatedItems)

        viewModelScope.launch {
            repository
                .triageItem(item.id, state)
                .onFailure {
                    val restored =
                        (_uiState.value as? LibraryUiState.Success)?.items?.toMutableList()
                            ?: mutableListOf()
                    restored.add(removedIndex.coerceAtMost(restored.size), item)
                    _uiState.value = (_uiState.value as? LibraryUiState.Success)
                        ?.copy(items = restored)
                        ?: LibraryUiState.Success(items = restored, hasMore = false)
                    _effects.emit(LibraryEffect.ShowSnackbar(UiMessage(Res.string.library_error_triage)))
                }
        }
    }

    fun deleteItem(item: LibraryItem) {
        val current = _uiState.value as? LibraryUiState.Success ?: return
        val updatedItems = current.items.toMutableList()
        val removedIndex = updatedItems.indexOfFirst { it.id == item.id }
        if (removedIndex == -1) return

        updatedItems.removeAt(removedIndex)
        _uiState.value = current.copy(items = updatedItems)

        viewModelScope.launch {
            repository
                .deleteItem(item.id)
                .onFailure {
                    val restored =
                        (_uiState.value as? LibraryUiState.Success)?.items?.toMutableList()
                            ?: mutableListOf()
                    restored.add(removedIndex.coerceAtMost(restored.size), item)
                    _uiState.value = (_uiState.value as? LibraryUiState.Success)
                        ?.copy(items = restored)
                        ?: LibraryUiState.Success(items = restored, hasMore = false)
                    _effects.emit(LibraryEffect.ShowSnackbar(UiMessage(Res.string.library_error_delete)))
                }
        }
    }

    /**
     * Counts are fetched independently of the list: a failure here must leave the rows
     * on screen, so it clears the header rather than surfacing an error state.
     */
    private fun loadCounts() {
        val scope = _scope.value
        if (scope !is LibraryScope.Triage) {
            _counts.value = null
            return
        }
        val triage = _triageFilter.value.apiValue
        viewModelScope.launch {
            val result = repository.scopeCounts(triage)
            // A response that lost the race to a newer scope must not overwrite it, in
            // either direction: a stale failure would blank counts the user can see.
            if (_scope.value !is LibraryScope.Triage || _triageFilter.value.apiValue != triage) return@launch
            _counts.value = result.getOrNull()
        }
    }

    private fun loadItems(
        append: Boolean = false,
        isRefresh: Boolean = false,
    ) {
        viewModelScope.launch {
            if (!append && !isRefresh) {
                _uiState.value = LibraryUiState.Loading
            }

            val cursor = if (append) nextCursor else null
            val result =
                when (val scope = _scope.value) {
                    is LibraryScope.Triage ->
                        repository.listItems(
                            triageState = _triageFilter.value.apiValue,
                            itemType = _contentTypeFilter.value.apiValue,
                            cursor = cursor,
                        )
                    is LibraryScope.Collection ->
                        repository.listCollectionItems(scope.id, cursor = cursor)
                    is LibraryScope.SmartList ->
                        repository.listSmartListItems(scope.id, cursor = cursor)
                }

            result
                .onSuccess { paginated ->
                    nextCursor = paginated.page.nextCursor
                    val newItems =
                        if (append) {
                            val existing = (_uiState.value as? LibraryUiState.Success)?.items ?: emptyList()
                            existing + paginated.data
                        } else {
                            paginated.data
                        }
                    _uiState.value =
                        LibraryUiState.Success(
                            items = newItems,
                            hasMore = paginated.page.hasMore,
                            isLoadingMore = false,
                            isRefreshing = false,
                        )
                }.onFailure {
                    if (append) {
                        val current = _uiState.value as? LibraryUiState.Success
                        if (current != null) {
                            _uiState.value = current.copy(isLoadingMore = false)
                        }
                        _effects.emit(LibraryEffect.ShowSnackbar(UiMessage(Res.string.library_error_load_more)))
                    } else {
                        _uiState.value = LibraryUiState.Error(UiMessage(Res.string.library_error_load))
                    }
                }
        }
    }
}
