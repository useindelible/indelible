package app.indelible.library.viewmodel

import app.indelible.core.i18n.UiMessage
import app.indelible.core.model.ItemDetail

sealed class ItemDetailUiState {
    data object Loading : ItemDetailUiState()

    data class Success(
        val item: ItemDetail,
    ) : ItemDetailUiState()

    data class Error(
        val message: UiMessage,
    ) : ItemDetailUiState()
}

sealed class ItemDetailEffect {
    data object NavigateBack : ItemDetailEffect()

    data class ShowSnackbar(
        val message: UiMessage,
    ) : ItemDetailEffect()
}
