package app.indelible.offline.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import app.indelible.core.offline.SessionRegistry
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn

/** One document's offline status, as the reader shows it. */
class DocumentOfflineViewModel(
    documentId: String,
    statuses: OfflineStatuses,
) : ViewModel() {
    val status: StateFlow<DocumentOfflineStatus> =
        statuses
            .observe()
            .map { it[documentId] ?: DocumentOfflineStatus() }
            .stateIn(viewModelScope, SharingStarted.WhileSubscribed(STOP_TIMEOUT_MS), DocumentOfflineStatus())

    private companion object {
        const val STOP_TIMEOUT_MS = 5_000L
    }
}
