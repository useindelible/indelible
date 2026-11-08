package app.indelible.offline.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import app.indelible.core.offline.CatalogEntry
import app.indelible.core.offline.DownloadManager
import app.indelible.core.offline.OfflineStore
import app.indelible.core.offline.OutboxKind
import app.indelible.core.offline.OutboxPayload
import app.indelible.core.offline.OutboxRow
import app.indelible.core.offline.OutboxState
import app.indelible.core.offline.Session
import app.indelible.core.offline.SessionRegistry
import app.indelible.core.storage.UserPreferencesStorage
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
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
    private val downloads: DownloadManager,
    private val preferences: UserPreferencesStorage,
    private val requestDrain: () -> Unit,
    private val unreachable: Flow<Boolean> = flowOf(false),
) : ViewModel() {
    private val capBytes = MutableStateFlow<Long?>(null)
    private val _notice = MutableStateFlow<StorageNotice?>(null)
    val notice: StateFlow<StorageNotice?> = _notice.asStateFlow()

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
            currentScope()?.let { downloads.enforceCap(it) }
        }
    }

    /** Retries every failed row of the change, so a document's grouped reading events retry together. */
    fun retry(changeId: String) {
        val scope = currentScope() ?: return
        val change = state.value.changes.firstOrNull { it.id == changeId }
        if (change?.canRetry == false) return
        val rowIds = change?.rowIds ?: listOf(changeId)
        viewModelScope.launch {
            val retried = rowIds.map { store.retryRow(scope, it) }
            if (retried.any { it }) requestDrain()
        }
    }

    /** Sends what is queued now, whatever backoff the rows were in. */
    fun retryNow() {
        val scope = currentScope() ?: return
        viewModelScope.launch {
            store.resetBackoff(scope)
            requestDrain()
        }
    }

    /** Deletes the change and the changes blocked behind it; nothing reaches the server. */
    fun discard(changeId: String) {
        val session = registry.current.value.session ?: return
        val change = state.value.changes.firstOrNull { it.id == changeId } ?: return
        val rowIds = change.rowIds + change.blocked.flatMap { it.rowIds }
        viewModelScope.launch { store.discardRows(session, rowIds) }
    }

    fun keepMine(changeId: String) {
        resolveNote(changeId, StorageNotice.KEPT_MINE) { session, id, seen ->
            store.keepLocalNote(session, id, seen).also { if (it) requestDrain() }
        }
    }

    fun keepOther(changeId: String) {
        resolveNote(changeId, StorageNotice.KEPT_OTHER, store::keepServerNote)
    }

    fun dismissNotice() {
        _notice.value = null
    }

    // The sheet showed one server version; a row that moved on since is refused and the list says why.
    private fun resolveNote(
        changeId: String,
        kept: StorageNotice,
        resolve: suspend (Session, String, Long) -> Boolean,
    ) {
        val session = registry.current.value.session ?: return
        val change = state.value.changes.firstOrNull { it.id == changeId }
        val seen = change?.failure?.serverNoteUpdatedAtEpochMs
        viewModelScope.launch {
            val done = seen != null && resolve(session, changeId, seen)
            _notice.value = if (done) kept else StorageNotice.CHANGED_REFRESHED
        }
    }

    /** Leaves the copy in place; only the mark that protects it from eviction goes. */
    fun unpin(documentId: String) {
        val scope = currentScope() ?: return
        viewModelScope.launch { downloads.unpin(scope, documentId) }
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

    private fun currentScope(): String? {
        val session = registry.current.value.session
        return session?.scope
    }

    private fun stateOf(scope: String): Flow<StorageUiState> =
        combine(
            store.observeCatalog(scope),
            store.observeOutbox(scope),
            capBytes.filterNotNull(),
            unreachable,
        ) { catalog, rows, cap, serverUnreachable ->
            StorageUiState(
                capBytes = cap,
                kept = catalog.filter { it.pinned },
                cached = catalog.filterNot { it.pinned },
                changes = pendingChanges(rows, catalog),
                serverUnreachable = serverUnreachable,
            )
        }

    private companion object {
        const val STOP_TIMEOUT_MS = 5_000L
    }
}

/**
 * The queue in send order, with each failed highlight create carrying the changes it blocks. A document's
 * reading events are one change at the place of the first, newest time and worst status, since a long
 * offline read queues thousands of them.
 */
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
        createdAt = row.createdAt,
        detail = row.payload.detail(),
        blocked = dependants,
        failure = row.failure,
    )
    val progress = rows.filter { it.isLiveProgress() }.groupBy { it.documentId }
    return rows.mapNotNull { row ->
        when {
            row.state == OutboxState.BLOCKED && row.entityId in failedCreates -> null
            row.isFailedCreate() -> change(row, blocked[row.entityId].orEmpty().map { change(it) })
            row.isLiveProgress() ->
                progress.getValue(row.documentId).takeIf { it.first() === row }?.progressChange(titles)
            else -> change(row)
        }
    }
}

private fun OutboxRow.isLiveProgress(): Boolean = kind == OutboxKind.READING_EVENT && !superseded

private fun List<OutboxRow>.progressChange(titles: Map<String, String?>): PendingChange =
    PendingChange(
        id = first().id,
        kind = OutboxKind.READING_EVENT,
        documentTitle = titles[first().documentId],
        status = map { it.status() }.minBy { STATUS_WEIGHT.indexOf(it) },
        error = lastOrNull { it.lastError != null }?.lastError,
        createdAt = maxOf { it.createdAt },
        rowIds = map { it.id },
        failure = firstOrNull { it.state == OutboxState.FAILED }?.failure,
    )

private val STATUS_WEIGHT =
    listOf(ChangeStatus.FAILED, ChangeStatus.BLOCKED, ChangeStatus.RETRYING, ChangeStatus.PENDING)

private fun OutboxRow.isFailedCreate(): Boolean = kind == OutboxKind.HIGHLIGHT_CREATE && state == OutboxState.FAILED

private fun OutboxRow.status(): ChangeStatus =
    when {
        superseded -> ChangeStatus.SUPERSEDED
        state == OutboxState.FAILED -> ChangeStatus.FAILED
        state == OutboxState.BLOCKED -> ChangeStatus.BLOCKED
        attempts > 0 -> ChangeStatus.RETRYING
        else -> ChangeStatus.PENDING
    }

private fun OutboxPayload.detail(): String? =
    when (this) {
        is OutboxPayload.HighlightCreate -> textContent
        is OutboxPayload.HighlightNote -> body
        is OutboxPayload.DocumentNote -> body
        else -> null
    }
