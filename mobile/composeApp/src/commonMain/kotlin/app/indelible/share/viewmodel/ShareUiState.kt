package app.indelible.share.viewmodel

import app.indelible.core.i18n.UiMessage
import app.indelible.core.model.SaveLibraryEntryResponse

sealed class ShareUiState {
    data object Idle : ShareUiState()

    data object Saving : ShareUiState()

    data class Success(
        val response: SaveLibraryEntryResponse,
    ) : ShareUiState()

    data object AlreadySaved : ShareUiState()

    data object Queued : ShareUiState()

    data object AuthRequired : ShareUiState()

    data class Error(
        val message: UiMessage,
    ) : ShareUiState()

    data object InvalidUrl : ShareUiState()
}
