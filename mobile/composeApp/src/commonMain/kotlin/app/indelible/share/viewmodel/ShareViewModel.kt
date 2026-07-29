package app.indelible.share.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import app.indelible.share.SaveResult
import app.indelible.share.SaveUrlUseCase
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

class ShareViewModel(
    private val saveUrlUseCase: SaveUrlUseCase,
) : ViewModel() {
    private val _uiState = MutableStateFlow<ShareUiState>(ShareUiState.Idle)
    val uiState: StateFlow<ShareUiState> = _uiState.asStateFlow()

    fun save(url: String) {
        if (_uiState.value is ShareUiState.Saving) return
        _uiState.value = ShareUiState.Saving
        viewModelScope.launch {
            _uiState.value =
                when (val result = saveUrlUseCase.save(url)) {
                    is SaveResult.Success -> ShareUiState.Success(result.response)
                    is SaveResult.AlreadySaved -> ShareUiState.AlreadySaved
                    is SaveResult.Queued -> ShareUiState.Queued
                    is SaveResult.AuthRequired -> ShareUiState.AuthRequired
                    is SaveResult.InvalidUrl -> ShareUiState.InvalidUrl
                    is SaveResult.Error -> ShareUiState.Error(result.message)
                }
        }
    }
}
