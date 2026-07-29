package app.indelible.library.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import app.indelible.library.repository.LibraryRepository
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
                .onFailure { error ->
                    _effects.emit(ItemDetailEffect.ShowSnackbar(error.message ?: "Failed to triage item"))
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
                .onFailure { error ->
                    _uiState.value = ItemDetailUiState.Success(current)
                    _effects.emit(ItemDetailEffect.ShowSnackbar(error.message ?: "Failed to update favorite"))
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
                .onFailure { error ->
                    _uiState.value = ItemDetailUiState.Success(current)
                    _effects.emit(ItemDetailEffect.ShowSnackbar(error.message ?: "Failed to update shortlist"))
                }
        }
    }

    fun rearchive() {
        viewModelScope.launch {
            repository
                .rearchiveItem(itemId)
                .onSuccess { updated -> _uiState.value = ItemDetailUiState.Success(updated) }
                .onFailure { error ->
                    _effects.emit(ItemDetailEffect.ShowSnackbar(error.message ?: "Failed to rearchive item"))
                }
        }
    }

    fun deleteItem() {
        viewModelScope.launch {
            repository
                .deleteItem(itemId)
                .onSuccess { _effects.emit(ItemDetailEffect.NavigateBack) }
                .onFailure { error ->
                    _effects.emit(ItemDetailEffect.ShowSnackbar(error.message ?: "Failed to delete item"))
                }
        }
    }

    private fun loadItem() {
        viewModelScope.launch {
            _uiState.value = ItemDetailUiState.Loading
            repository
                .getItem(itemId)
                .onSuccess { item -> _uiState.value = ItemDetailUiState.Success(item) }
                .onFailure { error ->
                    _uiState.value = ItemDetailUiState.Error(error.message ?: "Failed to load item")
                }
        }
    }
}
