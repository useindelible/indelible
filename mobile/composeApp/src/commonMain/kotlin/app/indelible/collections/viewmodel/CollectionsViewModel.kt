package app.indelible.collections.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import app.indelible.api.generated.models.CollectionResponse
import app.indelible.collections.repository.CollectionsRepository
import app.indelible.core.i18n.UiMessage
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.collections_error_load
import indelible.composeapp.generated.resources.collections_error_refresh
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

data class CollectionsState(
    val collections: List<CollectionResponse> = emptyList(),
    val isLoading: Boolean = false,
    val isRefreshing: Boolean = false,
    val isLoadingMore: Boolean = false,
    val hasMore: Boolean = false,
    val error: UiMessage? = null,
)

class CollectionsViewModel(
    private val repository: CollectionsRepository,
) : ViewModel() {
    private val _state = MutableStateFlow(CollectionsState())
    val state: StateFlow<CollectionsState> = _state.asStateFlow()

    private var nextCursor: String? = null

    init {
        load()
    }

    fun load() {
        if (_state.value.isLoading) return
        nextCursor = null
        _state.update { it.copy(isLoading = true, error = null) }
        viewModelScope.launch {
            repository
                .listCollections(cursor = null)
                .onSuccess { response ->
                    nextCursor = response.page.nextCursor
                    _state.update {
                        it.copy(
                            collections = response.data,
                            hasMore = response.page.hasMore,
                            isLoading = false,
                        )
                    }
                }.onFailure {
                    _state.update {
                        it.copy(isLoading = false, error = UiMessage(Res.string.collections_error_load))
                    }
                }
        }
    }

    fun refresh() {
        nextCursor = null
        _state.update { it.copy(isRefreshing = true, error = null) }
        viewModelScope.launch {
            repository
                .listCollections(cursor = null)
                .onSuccess { response ->
                    nextCursor = response.page.nextCursor
                    _state.update {
                        it.copy(
                            collections = response.data,
                            hasMore = response.page.hasMore,
                            isRefreshing = false,
                        )
                    }
                }.onFailure {
                    _state.update {
                        it.copy(isRefreshing = false, error = UiMessage(Res.string.collections_error_refresh))
                    }
                }
        }
    }

    fun loadNextPage() {
        val s = _state.value
        if (!s.hasMore || s.isLoadingMore) return
        _state.update { it.copy(isLoadingMore = true) }
        viewModelScope.launch {
            repository
                .listCollections(cursor = nextCursor)
                .onSuccess { response ->
                    nextCursor = response.page.nextCursor
                    _state.update {
                        it.copy(
                            collections = it.collections + response.data,
                            hasMore = response.page.hasMore,
                            isLoadingMore = false,
                        )
                    }
                }.onFailure {
                    _state.update { it.copy(isLoadingMore = false) }
                }
        }
    }
}
