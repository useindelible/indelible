package app.indelible.library.viewmodel

import app.indelible.core.model.ItemDetail

sealed class ItemDetailUiState {
    data object Loading : ItemDetailUiState()

    data class Success(
        val item: ItemDetail,
    ) : ItemDetailUiState()

    data class Error(
        val message: String,
    ) : ItemDetailUiState()
}

sealed class ItemDetailEffect {
    data object NavigateBack : ItemDetailEffect()

    data class ShowSnackbar(
        val message: String,
    ) : ItemDetailEffect()
}
