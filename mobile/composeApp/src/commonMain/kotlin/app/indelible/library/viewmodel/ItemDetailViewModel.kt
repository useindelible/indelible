package app.indelible.library.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import app.indelible.core.i18n.UiMessage
import app.indelible.library.repository.LibraryRepository
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.library_error_delete
import indelible.composeapp.generated.resources.library_error_favorite
import indelible.composeapp.generated.resources.library_error_load_item
import indelible.composeapp.generated.resources.library_error_rearchive
import indelible.composeapp.generated.resources.library_error_shortlist
import indelible.composeapp.generated.resources.library_error_triage
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

class ItemDetailViewModel(
    private val itemId: String,
    private val repository: LibraryRepository,
) : ViewModel() {
    private val _uiState = MutableStateFlow<ItemDetailUiState>(ItemDetailUiState.Loading)
    val uiState: StateFlow<ItemDetailUiState> = _uiState.asStateFlow()

    private val _effects = MutableSharedFlow<ItemDetailEffect>()
    val effects: SharedFlow<ItemDetailEffect> = _effects.asSharedFlow()

    init {
        loadItem()
    }

    fun triage(state: String) {
        viewModelScope.launch {
            repository
                .triageItem(itemId, state)
                .onSuccess { updated -> _uiState.value = ItemDetailUiState.Success(updated) }
                .onFailure {
                    _effects.emit(ItemDetailEffect.ShowSnackbar(UiMessage(Res.string.library_error_triage)))
                }
        }
    }

    fun toggleFavorite() {
        val current = (_uiState.value as? ItemDetailUiState.Success)?.item ?: return
        // Optimistic flip before the network call completes
        _uiState.value =
            ItemDetailUiState.Success(
                current.copy(isFavorite = !current.isFavorite),
            )
        viewModelScope.launch {
            repository
                .toggleFavorite(itemId)
                .onSuccess { updated -> _uiState.value = ItemDetailUiState.Success(updated) }
                .onFailure {
                    _uiState.value = ItemDetailUiState.Success(current)
                    _effects.emit(ItemDetailEffect.ShowSnackbar(UiMessage(Res.string.library_error_favorite)))
                }
        }
    }

    fun toggleShortlist() {
        val current = (_uiState.value as? ItemDetailUiState.Success)?.item ?: return
        _uiState.value =
            ItemDetailUiState.Success(
                current.copy(isShortlisted = !current.isShortlisted),
            )
        viewModelScope.launch {
            repository
                .toggleShortlist(itemId)
                .onSuccess { updated -> _uiState.value = ItemDetailUiState.Success(updated) }
                .onFailure {
                    _uiState.value = ItemDetailUiState.Success(current)
                    _effects.emit(ItemDetailEffect.ShowSnackbar(UiMessage(Res.string.library_error_shortlist)))
                }
        }
    }

    fun rearchive() {
        viewModelScope.launch {
            repository
                .rearchiveItem(itemId)
                .onSuccess { updated -> _uiState.value = ItemDetailUiState.Success(updated) }
                .onFailure {
                    _effects.emit(ItemDetailEffect.ShowSnackbar(UiMessage(Res.string.library_error_rearchive)))
                }
        }
    }

    fun deleteItem() {
        viewModelScope.launch {
            repository
                .deleteItem(itemId)
                .onSuccess { _effects.emit(ItemDetailEffect.NavigateBack) }
                .onFailure {
                    _effects.emit(ItemDetailEffect.ShowSnackbar(UiMessage(Res.string.library_error_delete)))
                }
        }
    }

    private fun loadItem() {
        viewModelScope.launch {
            _uiState.value = ItemDetailUiState.Loading
            repository
                .getItem(itemId)
                .onSuccess { item -> _uiState.value = ItemDetailUiState.Success(item) }
                .onFailure {
                    _uiState.value = ItemDetailUiState.Error(UiMessage(Res.string.library_error_load_item))
                }
        }
    }
}
