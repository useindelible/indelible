package app.indelible.feed.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import app.indelible.feed.model.FeedSubscription
import app.indelible.feed.repository.FeedRepository
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

sealed class AddFeedUiState {
    data object Idle : AddFeedUiState()

    data object Loading : AddFeedUiState()

    data object OpmlLoading : AddFeedUiState()

    data class Success(
        val subscription: FeedSubscription,
    ) : AddFeedUiState()

    data class Error(
        val message: String,
    ) : AddFeedUiState()
}

sealed class AddFeedEffect {
    data class ShowSnackbar(
        val message: String,
    ) : AddFeedEffect()

    data object NavigateBack : AddFeedEffect()
}

class AddFeedViewModel(
    private val repository: FeedRepository,
) : ViewModel() {
    private val _uiState = MutableStateFlow<AddFeedUiState>(AddFeedUiState.Idle)
    val uiState: StateFlow<AddFeedUiState> = _uiState.asStateFlow()

    private val _effects = MutableSharedFlow<AddFeedEffect>()
    val effects: SharedFlow<AddFeedEffect> = _effects.asSharedFlow()

    fun subscribe(url: String) {
        if (url.isBlank()) return
        _uiState.value = AddFeedUiState.Loading

        viewModelScope.launch {
            repository
                .subscribe(url.trim(), null)
                .onSuccess { subscription ->
                    _uiState.value = AddFeedUiState.Success(subscription)
                    _effects.emit(AddFeedEffect.NavigateBack)
                }.onFailure { error ->
                    _uiState.value = AddFeedUiState.Idle
                    _effects.emit(
                        AddFeedEffect.ShowSnackbar(error.message ?: "Failed to subscribe"),
                    )
                }
        }
    }

    fun importOpml(
        fileBytes: ByteArray,
        fileName: String,
    ) {
        _uiState.value = AddFeedUiState.OpmlLoading
        viewModelScope.launch {
            repository
                .importOpml(fileBytes, fileName)
                .onSuccess { result ->
                    _uiState.value = AddFeedUiState.Idle
                    val message =
                        buildString {
                            append("Imported ${result.created} feed")
                            if (result.created != 1) append("s")
                            if (result.skipped > 0) append(", ${result.skipped} skipped")
                            if (result.errors.isNotEmpty()) append(", ${result.errors.size} error(s)")
                        }
                    _effects.emit(AddFeedEffect.ShowSnackbar(message))
                }.onFailure { error ->
                    _uiState.value = AddFeedUiState.Idle
                    _effects.emit(AddFeedEffect.ShowSnackbar(error.message ?: "Failed to import OPML"))
                }
        }
    }
}
