package app.indelible.profile.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import app.indelible.profile.repository.AddLibraryRepository
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

sealed class AddLibraryUiState {
    data object Idle : AddLibraryUiState()

    data object Loading : AddLibraryUiState()
}

sealed class AddLibraryEffect {
    data class ShowSnackbar(
        val message: String,
    ) : AddLibraryEffect()

    data object NavigateBack : AddLibraryEffect()
}

class AddLibraryViewModel(
    private val repository: AddLibraryRepository,
) : ViewModel() {
    private val _uiState = MutableStateFlow<AddLibraryUiState>(AddLibraryUiState.Idle)
    val uiState: StateFlow<AddLibraryUiState> = _uiState.asStateFlow()

    private val _effects = MutableSharedFlow<AddLibraryEffect>()
    val effects: SharedFlow<AddLibraryEffect> = _effects.asSharedFlow()

    fun save(url: String) {
        if (url.isBlank()) return
        _uiState.value = AddLibraryUiState.Loading
        viewModelScope.launch {
            repository
                .save(url.trim())
                .onSuccess {
                    _uiState.value = AddLibraryUiState.Idle
                    _effects.emit(AddLibraryEffect.NavigateBack)
                }.onFailure { error ->
                    _uiState.value = AddLibraryUiState.Idle
                    _effects.emit(AddLibraryEffect.ShowSnackbar(error.message ?: "Failed to save"))
                }
        }
    }
}
