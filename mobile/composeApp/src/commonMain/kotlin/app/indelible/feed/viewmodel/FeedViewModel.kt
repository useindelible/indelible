package app.indelible.feed.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import app.indelible.core.i18n.UiMessage
import app.indelible.feed.model.FeedItemWithState
import app.indelible.feed.model.PaginatedFeedItems
import app.indelible.feed.repository.FeedRepository
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.feed_error_load
import indelible.composeapp.generated.resources.feed_error_load_more
import indelible.composeapp.generated.resources.feed_error_mark_all_seen
import indelible.composeapp.generated.resources.feed_error_mark_seen
import indelible.composeapp.generated.resources.feed_error_open
import indelible.composeapp.generated.resources.feed_error_save
import indelible.composeapp.generated.resources.feed_status_all_seen
import indelible.composeapp.generated.resources.feed_status_saved_library
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

class FeedViewModel(
    private val repository: FeedRepository,
) : ViewModel() {
    private val _uiState = MutableStateFlow<FeedUiState>(FeedUiState.Loading)
    val uiState: StateFlow<FeedUiState> = _uiState.asStateFlow()

    private val _effects = MutableSharedFlow<FeedEffect>()
    val effects: SharedFlow<FeedEffect> = _effects.asSharedFlow()

    private val _feedFilter = MutableStateFlow(FeedFilter.UNSEEN)
    val feedFilter: StateFlow<FeedFilter> = _feedFilter.asStateFlow()

    private var nextCursor: String? = null
    private var cachedSourceCount: Int? = null
    private var cachedSourceCountExact: Boolean = true

    fun setFeedFilter(filter: FeedFilter) {
        _feedFilter.value = filter
        nextCursor = null
        loadItems()
    }

    fun refresh() {
        val current = _uiState.value
        if (current is FeedUiState.Success) {
            _uiState.value = current.copy(isRefreshing = true)
        }
        nextCursor = null
        loadItems(isRefresh = true)
    }

    fun loadNextPage() {
        val current = _uiState.value as? FeedUiState.Success ?: return
        if (!current.hasMore || current.isLoadingMore) return
        _uiState.value = current.copy(isLoadingMore = true)
        loadItems(append = true)
    }

    fun openDelivery(deliveryId: String) {
        viewModelScope.launch {
            repository
                .prepareDelivery(deliveryId)
                .onSuccess { documentId ->
                    // Preparation marks the delivery seen server-side; drop it from the unseen
                    // list so the row doesn't linger stale when the reader is dismissed.
                    if (_feedFilter.value == FeedFilter.UNSEEN) {
                        val current = _uiState.value as? FeedUiState.Success
                        if (current != null) {
                            _uiState.value =
                                current.copy(items = current.items.filterNot { it.id == deliveryId })
                        }
                    }
                    _effects.emit(FeedEffect.OpenReader(documentId))
                }.onFailure {
                    _effects.emit(FeedEffect.ShowSnackbar(UiMessage(Res.string.feed_error_open)))
                }
        }
    }

    fun markSeen(item: FeedItemWithState) {
        val current = _uiState.value as? FeedUiState.Success ?: return
        val updatedItems = current.items.toMutableList()
        val removedIndex = updatedItems.indexOfFirst { it.id == item.id }
        if (removedIndex == -1) return

        updatedItems.removeAt(removedIndex)
        _uiState.value = current.copy(items = updatedItems)

        viewModelScope.launch {
            repository
                .markSeen(item.id)
                .onFailure {
                    val restored =
                        (_uiState.value as? FeedUiState.Success)?.items?.toMutableList()
                            ?: mutableListOf()
                    restored.add(removedIndex.coerceAtMost(restored.size), item)
                    _uiState.value = (_uiState.value as? FeedUiState.Success)
                        ?.copy(items = restored)
                        ?: FeedUiState.Success(items = restored, hasMore = false)
                    _effects.emit(FeedEffect.ShowSnackbar(UiMessage(Res.string.feed_error_mark_seen)))
                }
        }
    }

    fun saveToLibrary(item: FeedItemWithState) {
        val current = _uiState.value as? FeedUiState.Success ?: return
        _uiState.value = current.copy(savedItemIds = current.savedItemIds + item.id)

        viewModelScope.launch {
            repository
                .saveToLibrary(item.id)
                .onSuccess {
                    _effects.emit(FeedEffect.ShowSnackbar(UiMessage(Res.string.feed_status_saved_library)))
                }.onFailure {
                    val latest = _uiState.value as? FeedUiState.Success
                    if (latest != null) {
                        _uiState.value = latest.copy(savedItemIds = latest.savedItemIds - item.id)
                    }
                    _effects.emit(FeedEffect.ShowSnackbar(UiMessage(Res.string.feed_error_save)))
                }
        }
    }

    fun markAllSeen() {
        val current = _uiState.value as? FeedUiState.Success ?: return
        val previousItems = current.items.toList()
        _uiState.value = current.copy(items = emptyList())

        viewModelScope.launch {
            repository
                .markAllSeen()
                .onSuccess {
                    _effects.emit(FeedEffect.ShowSnackbar(UiMessage(Res.string.feed_status_all_seen)))
                }.onFailure {
                    _uiState.value = (_uiState.value as? FeedUiState.Success)
                        ?.copy(items = previousItems)
                        ?: FeedUiState.Success(items = previousItems, hasMore = false)
                    _effects.emit(FeedEffect.ShowSnackbar(UiMessage(Res.string.feed_error_mark_all_seen)))
                }
        }
    }

    private fun loadItems(
        append: Boolean = false,
        isRefresh: Boolean = false,
    ) {
        viewModelScope.launch {
            if (!append && !isRefresh) {
                _uiState.value = FeedUiState.Loading
            }

            repository
                .listItems(
                    state = _feedFilter.value.apiValue,
                    cursor = if (append) nextCursor else null,
                ).onSuccess { paginated ->
                    _uiState.value = buildSuccessState(paginated, append, isRefresh)
                }.onFailure {
                    handleLoadFailure(append)
                }
        }
    }

    private suspend fun buildSuccessState(
        paginated: PaginatedFeedItems,
        append: Boolean,
        isRefresh: Boolean,
    ): FeedUiState.Success {
        nextCursor = paginated.page.nextCursor
        val existingState = _uiState.value as? FeedUiState.Success
        val newItems =
            if (append) (existingState?.items ?: emptyList()) + paginated.data else paginated.data

        val (sourceCount, sourceCountExact) =
            if (append) {
                (existingState?.sourceCount ?: cachedSourceCount) to
                    (existingState?.sourceCountExact ?: cachedSourceCountExact)
            } else {
                ensureSourceCount(force = isRefresh)
            }

        return FeedUiState.Success(
            items = newItems,
            hasMore = paginated.page.hasMore,
            isLoadingMore = false,
            isRefreshing = false,
            hasSubscriptions = sourceCount?.let { it > 0 } ?: true,
            savedItemIds = if (append) existingState?.savedItemIds ?: emptySet() else emptySet(),
            sourceCount = sourceCount,
            sourceCountExact = sourceCountExact,
        )
    }

    private suspend fun handleLoadFailure(append: Boolean) {
        if (!append) {
            _uiState.value = FeedUiState.Error(UiMessage(Res.string.feed_error_load))
            return
        }
        (_uiState.value as? FeedUiState.Success)?.let { current ->
            _uiState.value = current.copy(isLoadingMore = false)
        }
        _effects.emit(FeedEffect.ShowSnackbar(UiMessage(Res.string.feed_error_load_more)))
    }

    private suspend fun ensureSourceCount(force: Boolean): Pair<Int?, Boolean> {
        if (!force && cachedSourceCount != null) {
            return cachedSourceCount to cachedSourceCountExact
        }
        return repository
            .listSubscriptions(limit = SOURCE_COUNT_PAGE_SIZE)
            .map { paginated ->
                cachedSourceCount = paginated.data.size
                cachedSourceCountExact = !paginated.page.hasMore
                cachedSourceCount to cachedSourceCountExact
            }.getOrDefault(cachedSourceCount to cachedSourceCountExact)
    }

    private companion object {
        const val SOURCE_COUNT_PAGE_SIZE = 50
    }
}
