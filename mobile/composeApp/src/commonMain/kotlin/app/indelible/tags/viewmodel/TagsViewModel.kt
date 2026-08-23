package app.indelible.tags.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import app.indelible.core.i18n.UiMessage
import app.indelible.reader.model.TagData
import app.indelible.tags.repository.TagsRepository
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.tags_error_load
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

enum class TagScope(
    val apiValue: String?,
) {
    ALL(null),
    DOC("document"),
    HIGHLIGHT("highlight"),
}

data class TagsState(
    val tags: List<TagData> = emptyList(),
    val filter: String = "",
    val scope: TagScope = TagScope.ALL,
    val isLoading: Boolean = false,
    val isRefreshing: Boolean = false,
    val error: UiMessage? = null,
)

class TagsViewModel(
    private val repository: TagsRepository,
) : ViewModel() {
    private val _state = MutableStateFlow(TagsState())
    val state: StateFlow<TagsState> = _state.asStateFlow()

    init {
        load()
    }

    fun load() {
        _state.update { it.copy(isLoading = true, error = null) }
        viewModelScope.launch {
            repository
                .listTags(scope = _state.value.scope.apiValue)
                .onSuccess { tags ->
                    _state.update {
                        it.copy(
                            tags = tags.sortedBy { tag -> tag.name.lowercase() },
                            isLoading = false,
                        )
                    }
                }.onFailure {
                    _state.update { it.copy(isLoading = false, error = UiMessage(Res.string.tags_error_load)) }
                }
        }
    }

    fun refresh() {
        _state.update { it.copy(isRefreshing = true, error = null) }
        viewModelScope.launch {
            repository
                .listTags(scope = _state.value.scope.apiValue)
                .onSuccess { tags ->
                    _state.update {
                        it.copy(
                            tags = tags.sortedBy { tag -> tag.name.lowercase() },
                            isRefreshing = false,
                        )
                    }
                }.onFailure {
                    _state.update { it.copy(isRefreshing = false) }
                }
        }
    }

    fun setFilter(filter: String) {
        _state.update { it.copy(filter = filter) }
    }

    // Clicking the active segment deselects it (→ ALL), matching web behavior.
    fun toggleScope(tapped: TagScope) {
        val newScope = if (_state.value.scope == tapped) TagScope.ALL else tapped
        _state.update { it.copy(scope = newScope) }
        load()
    }
}
