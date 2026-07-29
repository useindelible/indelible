package app.indelible.sidebar.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import app.indelible.sidebar.repository.SidebarRepository
import kotlinx.coroutines.async
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

class SidebarViewModel(
    private val repository: SidebarRepository,
) : ViewModel() {
    private val _uiState = MutableStateFlow<SidebarUiState>(SidebarUiState.Loading)
    val uiState: StateFlow<SidebarUiState> = _uiState.asStateFlow()

    // Collections and smart lists load in parallel. Either fetch failing degrades to an
    // empty list rather than an error screen: the drawer must stay navigable offline.
    fun load() {
        _uiState.value = SidebarUiState.Loading
        viewModelScope.launch {
            val collections = async { repository.listCollections() }
            val smartLists = async { repository.listSmartLists() }
            _uiState.value =
                SidebarUiState.Ready(
                    collections = collections.await().getOrDefault(emptyList()),
                    smartLists = smartLists.await().getOrDefault(emptyList()),
                )
        }
    }
}
