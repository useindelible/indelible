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

/** One document's offline status, and keeping or removing its copy. */
class DocumentOfflineViewModel(
    private val documentId: String,
    private val registry: SessionRegistry,
    statuses: OfflineStatuses,
    private val downloads: DownloadManager,
) : ViewModel() {
    val status: StateFlow<DocumentOfflineStatus> =
        statuses
            .observe()
            .map { it[documentId] ?: DocumentOfflineStatus() }
            .stateIn(viewModelScope, SharingStarted.WhileSubscribed(STOP_TIMEOUT_MS), DocumentOfflineStatus())

    /** Turning it off only unpins: the copy stays until the cap evicts it or the user removes it. */
    fun setKeepOffline(keep: Boolean) {
        val session = registry.current.value.session ?: return
        viewModelScope.launch {
            if (keep) downloads.keepOffline(session, documentId) else downloads.unpin(session.scope, documentId)
        }
    }

    fun removeFromDevice() {
        val session = registry.current.value.session ?: return
        viewModelScope.launch { downloads.removeFromDevice(session.scope, documentId) }
    }

    private companion object {
        const val STOP_TIMEOUT_MS = 5_000L
    }
}
