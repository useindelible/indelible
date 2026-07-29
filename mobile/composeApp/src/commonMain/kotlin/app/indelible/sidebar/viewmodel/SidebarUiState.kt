package app.indelible.sidebar.viewmodel

import app.indelible.sidebar.model.Collection
import app.indelible.sidebar.model.SmartList

sealed class SidebarUiState {
    data object Loading : SidebarUiState()

    data class Ready(
        val collections: List<Collection>,
        val smartLists: List<SmartList>,
    ) : SidebarUiState()
}
