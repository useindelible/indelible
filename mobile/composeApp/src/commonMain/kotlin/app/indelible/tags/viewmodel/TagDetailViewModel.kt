package app.indelible.tags.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import app.indelible.core.i18n.UiMessage
import app.indelible.core.model.LibraryItem
import app.indelible.reader.model.TagData
import app.indelible.tags.repository.TagsRepository
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.tags_error_load_detail
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

data class TagDetailState(
    val tag: TagData? = null,
    val allTags: List<TagData> = emptyList(),
    val children: List<TagData> = emptyList(),
    val items: List<LibraryItem> = emptyList(),
    val isLoading: Boolean = false,
    val isRefreshing: Boolean = false,
    val isLoadingMoreItems: Boolean = false,
    val hasMoreItems: Boolean = false,
    val error: UiMessage? = null,
)

class TagDetailViewModel(
    private val tagId: String,
    private val repository: TagsRepository,
) : ViewModel() {
    private val _state = MutableStateFlow(TagDetailState())
    val state: StateFlow<TagDetailState> = _state.asStateFlow()

    private var itemCursor: String? = null

    init {
        load()
    }

    fun load() {
        _state.update { it.copy(isLoading = true, error = null) }
        viewModelScope.launch {
            val tagResult = repository.getTag(tagId)
            val allTagsResult = repository.listTags()
            val itemsResult = repository.listTagItems(tagId, cursor = null)

            val children =
                allTagsResult
                    .getOrNull()
                    ?.filter { it.parentId == tagId }
                    ?.sortedBy { it.name.lowercase() }
                    ?: emptyList()

            itemCursor = itemsResult.getOrNull()?.page?.nextCursor

            _state.update {
                it.copy(
                    tag = tagResult.getOrNull() ?: it.tag,
                    allTags = allTagsResult.getOrNull() ?: it.allTags,
                    children = children,
                    items = itemsResult.getOrNull()?.data ?: it.items,
                    hasMoreItems = itemsResult.getOrNull()?.page?.hasMore ?: false,
                    isLoading = false,
                    error =
                        if (tagResult.isFailure) {
                            UiMessage(Res.string.tags_error_load_detail)
                        } else {
                            null
                        },
                )
            }
        }
    }

    fun refresh() {
        _state.update { it.copy(isRefreshing = true, error = null) }
        viewModelScope.launch {
            val tagResult = repository.getTag(tagId)
            val allTagsResult = repository.listTags()
            val itemsResult = repository.listTagItems(tagId, cursor = null)

            val children =
                allTagsResult
                    .getOrNull()
                    ?.filter { it.parentId == tagId }
                    ?.sortedBy { it.name.lowercase() }
                    ?: emptyList()

            itemCursor = itemsResult.getOrNull()?.page?.nextCursor

            _state.update {
                it.copy(
                    tag = tagResult.getOrNull() ?: it.tag,
                    allTags = allTagsResult.getOrNull() ?: it.allTags,
                    children = children,
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
                .listTagItems(tagId, cursor = itemCursor)
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
