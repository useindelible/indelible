package app.indelible.trash.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import app.indelible.core.model.LibraryItem
import app.indelible.trash.repository.TrashRepository
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

data class TrashState(
    val items: List<LibraryItem> = emptyList(),
    val isLoading: Boolean = false,
    val isRefreshing: Boolean = false,
    val isLoadingMore: Boolean = false,
    val hasMore: Boolean = false,
    val error: String? = null,
    val showEmptyConfirm: Boolean = false,
    val isEmptying: Boolean = false,
    val restoringItemIds: Set<String> = emptySet(),
)

class TrashViewModel(
    private val repository: TrashRepository,
) : ViewModel() {
    private val _state = MutableStateFlow(TrashState())
    val state: StateFlow<TrashState> = _state.asStateFlow()

    private var cursor: String? = null

    init {
        load()
    }

    fun load() {
        _state.update { it.copy(isLoading = true, error = null) }
        viewModelScope.launch {
            repository
                .listTrash(cursor = null)
                .onSuccess { response ->
                    cursor = response.page.nextCursor
                    _state.update {
                        it.copy(
                            items = response.data,
                            hasMore = response.page.hasMore,
                            isLoading = false,
                        )
                    }
                }.onFailure { e ->
                    _state.update { it.copy(isLoading = false, error = e.message) }
                }
        }
    }

    fun refresh() {
        _state.update { it.copy(isRefreshing = true, error = null) }
        viewModelScope.launch {
            repository
                .listTrash(cursor = null)
                .onSuccess { response ->
                    cursor = response.page.nextCursor
                    _state.update {
                        it.copy(
                            items = response.data,
                            hasMore = response.page.hasMore,
                            isRefreshing = false,
                        )
                    }
                }.onFailure {
                    _state.update { it.copy(isRefreshing = false) }
                }
        }
    }

    fun loadNextPage() {
        val s = _state.value
        if (!s.hasMore || s.isLoadingMore) return
        _state.update { it.copy(isLoadingMore = true) }
        viewModelScope.launch {
            repository
                .listTrash(cursor = cursor)
                .onSuccess { response ->
                    cursor = response.page.nextCursor
                    _state.update {
                        it.copy(
                            items = it.items + response.data,
                            hasMore = response.page.hasMore,
                            isLoadingMore = false,
                        )
                    }
                }.onFailure {
                    _state.update { it.copy(isLoadingMore = false) }
                }
        }
    }

    fun restoreItem(itemId: String) {
        _state.update { it.copy(restoringItemIds = it.restoringItemIds + itemId) }
        viewModelScope.launch {
            repository
                .restoreItem(itemId)
                .onSuccess {
                    _state.update {
                        it.copy(
                            items = it.items.filter { item -> item.id != itemId },
                            restoringItemIds = it.restoringItemIds - itemId,
                        )
                    }
                }.onFailure {
                    _state.update { it.copy(restoringItemIds = it.restoringItemIds - itemId) }
                }
        }
    }

    fun requestEmptyTrash() {
        _state.update { it.copy(showEmptyConfirm = true) }
    }

    fun dismissEmptyConfirm() {
        _state.update { it.copy(showEmptyConfirm = false) }
    }

    fun emptyTrash() {
        _state.update { it.copy(showEmptyConfirm = false, isEmptying = true) }
        viewModelScope.launch {
            repository
                .emptyTrash()
                .onSuccess {
                    cursor = null
                    _state.update {
                        it.copy(
                            items = emptyList(),
                            hasMore = false,
                            isEmptying = false,
                        )
                    }
                }.onFailure {
                    _state.update { it.copy(isEmptying = false) }
                }
        }
    }
}
