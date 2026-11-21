package app.indelible.offline.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import app.indelible.core.offline.DownloadManager
import app.indelible.core.offline.SessionRegistry
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch

/** One document's offline status, as the reader shows it, and the one action the reader keeps: Keep. */
class DocumentOfflineViewModel(
    private val documentId: String,
    statuses: OfflineStatuses,
    private val registry: SessionRegistry,
    private val downloads: DownloadManager,
) : ViewModel() {
    val status: StateFlow<DocumentOfflineStatus> =
        statuses
            .observe()
            .map { it[documentId] ?: DocumentOfflineStatus() }
            .stateIn(viewModelScope, SharingStarted.WhileSubscribed(STOP_TIMEOUT_MS), DocumentOfflineStatus())

    /** Pins the copy, or fetches a pinned one; a pinned fetch ignores the automatic cap, as the user chose it. */
    fun keep() {
        val session = registry.current.value.session ?: return
        viewModelScope.launch { downloads.keepOffline(session, documentId) }
    }

    private companion object {
        const val STOP_TIMEOUT_MS = 5_000L
    }
}
