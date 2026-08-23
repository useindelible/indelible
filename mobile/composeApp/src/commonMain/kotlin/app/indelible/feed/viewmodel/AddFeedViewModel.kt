package app.indelible.feed.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import app.indelible.core.i18n.UiMessage
import app.indelible.feed.model.FeedSubscription
import app.indelible.feed.repository.FeedRepository
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.feed_error_import_opml
import indelible.composeapp.generated.resources.feed_error_subscribe
import indelible.composeapp.generated.resources.feed_opml_import_result
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
        val message: UiMessage,
    ) : AddFeedUiState()
}

sealed class AddFeedEffect {
    data class ShowSnackbar(
        val message: UiMessage,
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
                }.onFailure {
                    _uiState.value = AddFeedUiState.Idle
                    _effects.emit(
                        AddFeedEffect.ShowSnackbar(UiMessage(Res.string.feed_error_subscribe)),
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
                    _effects.emit(
                        AddFeedEffect.ShowSnackbar(
                            UiMessage(
                                Res.string.feed_opml_import_result,
                                listOf(result.created, result.skipped, result.errors.size),
                            ),
                        ),
                    )
                }.onFailure {
                    _uiState.value = AddFeedUiState.Idle
                    _effects.emit(AddFeedEffect.ShowSnackbar(UiMessage(Res.string.feed_error_import_opml)))
                }
        }
    }
}
