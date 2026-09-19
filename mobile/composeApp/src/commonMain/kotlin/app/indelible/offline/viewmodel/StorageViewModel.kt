package app.indelible.offline.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import app.indelible.core.offline.CatalogEntry
import app.indelible.core.offline.DownloadManager
import app.indelible.core.offline.OfflineCopies
import app.indelible.core.offline.OfflineStore
import app.indelible.core.offline.OutboxKind
import app.indelible.core.offline.OutboxRow
import app.indelible.core.offline.OutboxState
import app.indelible.core.offline.SessionRegistry
import app.indelible.core.storage.UserPreferencesStorage
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import okio.IOException

/** What the signed-in account keeps on this device and what it still has to send. */
class StorageViewModel(
    private val registry: SessionRegistry,
    private val store: OfflineStore,
    private val copies: OfflineCopies,
    private val downloads: DownloadManager,
    private val preferences: UserPreferencesStorage,
    private val requestDrain: () -> Unit,
) : ViewModel() {
    private val capBytes = MutableStateFlow<Long?>(null)

    @OptIn(ExperimentalCoroutinesApi::class)
    val state: StateFlow<StorageUiState> =
        registry.current
            .map { it.session?.scope }
            .distinctUntilChanged()
            .flatMapLatest { scope -> scope?.let(::stateOf) ?: flowOf(StorageUiState()) }
            .stateIn(viewModelScope, SharingStarted.WhileSubscribed(STOP_TIMEOUT_MS), StorageUiState())

    init {
        viewModelScope.launch { capBytes.value = preferences.getOfflineCapBytes() }
    }

    fun setCap(bytes: Long) {
        capBytes.value = bytes
        viewModelScope.launch {
            preferences.saveOfflineCapBytes(bytes)
            currentScope()?.let { copies.enforceCap(it, keep = null) }
        }
    }

    fun retry(changeId: String) {
        val scope = currentScope() ?: return
        viewModelScope.launch { if (store.retryRow(scope, changeId)) requestDrain() }
    }

    fun removeCopy(documentId: String) {
        val scope = currentScope() ?: return
        viewModelScope.launch { downloads.removeFromDevice(scope, documentId) }
    }

    fun removeDownloads() {
        val scope = currentScope() ?: return
        viewModelScope.launch {
            try {
                downloads.removeAllDownloads(scope)
            } catch (ignored: IOException) {
                // The copies' rows are gone; the launch sweep reclaims the directory left behind.
            }
        }
    }

    private fun currentScope(): String? = registry.current.value.session?.scope

    private fun stateOf(scope: String): Flow<StorageUiState> =
        combine(
            store.observeCatalog(scope),
            store.observeOutbox(scope),
            capBytes.filterNotNull(),
        ) { catalog, rows, cap ->
            StorageUiState(
                capBytes = cap,
                kept = catalog.filter { it.pinned },
                cached = catalog.filterNot { it.pinned },
                changes = pendingChanges(rows, catalog),
            )
        }

    private companion object {
        const val STOP_TIMEOUT_MS = 5_000L
    }
}

/** The queue in send order, with each failed highlight create carrying the changes it blocks. */
internal fun pendingChanges(
    rows: List<OutboxRow>,
    catalog: List<CatalogEntry>,
): List<PendingChange> {
    val titles = catalog.associate { it.documentId to it.title }
    val failedCreates = rows.filter { it.isFailedCreate() }.map { it.entityId }.toSet()
    val blocked = rows.filter { it.state == OutboxState.BLOCKED }.groupBy { it.entityId }

    fun change(
        row: OutboxRow,
        dependants: List<PendingChange> = emptyList(),
    ) = PendingChange(
        id = row.id,
        kind = row.kind,
        documentTitle = titles[row.documentId],
        status = row.status(),
        error = row.lastError,
        blocked = dependants,
    )
    return rows.mapNotNull { row ->
        when {
            row.state == OutboxState.BLOCKED && row.entityId in failedCreates -> null
            row.isFailedCreate() -> change(row, blocked[row.entityId].orEmpty().map { change(it) })
            else -> change(row)
        }
    }
}

private fun OutboxRow.isFailedCreate(): Boolean = kind == OutboxKind.HIGHLIGHT_CREATE && state == OutboxState.FAILED

private fun OutboxRow.status(): ChangeStatus =
    when {
        superseded -> ChangeStatus.SUPERSEDED
        state == OutboxState.FAILED -> ChangeStatus.FAILED
        state == OutboxState.BLOCKED -> ChangeStatus.BLOCKED
        attempts > 0 -> ChangeStatus.RETRYING
        else -> ChangeStatus.PENDING
    }
