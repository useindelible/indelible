package app.indelible.collections.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import app.indelible.api.generated.models.CollectionResponse
import app.indelible.collections.repository.CollectionsRepository
import app.indelible.core.model.LibraryItem
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

data class CollectionDetailState(
    val collection: CollectionResponse? = null,
    val children: List<CollectionResponse> = emptyList(),
    val items: List<LibraryItem> = emptyList(),
    val isLoading: Boolean = false,
    val isRefreshing: Boolean = false,
    val isLoadingMoreItems: Boolean = false,
    val hasMoreItems: Boolean = false,
    val error: String? = null,
)

class CollectionDetailViewModel(
    private val collectionId: String,
    private val repository: CollectionsRepository,
) : ViewModel() {
    private val _state = MutableStateFlow(CollectionDetailState())
    val state: StateFlow<CollectionDetailState> = _state.asStateFlow()

    private var itemCursor: String? = null

    init {
        load()
    }

    fun load() {
        _state.update { it.copy(isLoading = true, error = null) }
        viewModelScope.launch {
            val collectionResult = repository.getCollection(collectionId)
            val childrenResult = repository.listCollectionChildren(collectionId)
            val itemsResult = repository.listCollectionItems(collectionId, cursor = null)

            val error =
                collectionResult.exceptionOrNull()?.message
                    ?: childrenResult.exceptionOrNull()?.message
                    ?: itemsResult.exceptionOrNull()?.message

            itemCursor = itemsResult.getOrNull()?.page?.nextCursor

            _state.update {
                it.copy(
                    collection = collectionResult.getOrNull() ?: it.collection,
                    children = childrenResult.getOrNull()?.data?.sortedBy { it.createdAt } ?: it.children,
                    items = itemsResult.getOrNull()?.data ?: it.items,
                    hasMoreItems = itemsResult.getOrNull()?.page?.hasMore ?: false,
                    isLoading = false,
                    error = if (collectionResult.isFailure) error else null,
                )
            }
        }
    }

    fun refresh() {
        _state.update { it.copy(isRefreshing = true, error = null) }
        viewModelScope.launch {
            val collectionResult = repository.getCollection(collectionId)
            val childrenResult = repository.listCollectionChildren(collectionId)
            val itemsResult = repository.listCollectionItems(collectionId, cursor = null)

            itemCursor = itemsResult.getOrNull()?.page?.nextCursor

            _state.update {
                it.copy(
                    collection = collectionResult.getOrNull() ?: it.collection,
                    children = childrenResult.getOrNull()?.data?.sortedBy { it.createdAt } ?: it.children,
                    items = itemsResult.getOrNull()?.data ?: it.items,
                    hasMoreItems = itemsResult.getOrNull()?.page?.hasMore ?: false,
                    isRefreshing = false,
                )
            }
        }
    }

    fun loadNextItemsPage() {
        val s = _state.value
        if (!s.hasMoreItems || s.isLoadingMoreItems) return
        _state.update { it.copy(isLoadingMoreItems = true) }
        viewModelScope.launch {
            repository
                .listCollectionItems(collectionId, cursor = itemCursor)
                .onSuccess { response ->
                    itemCursor = response.page.nextCursor
                    _state.update {
                        it.copy(
                            items = it.items + response.data,
                            hasMoreItems = response.page.hasMore,
                            isLoadingMoreItems = false,
                        )
                    }
                }.onFailure {
                    _state.update { it.copy(isLoadingMoreItems = false) }
                }
        }
    }
}
