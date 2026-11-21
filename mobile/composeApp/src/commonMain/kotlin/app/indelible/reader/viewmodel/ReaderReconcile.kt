package app.indelible.reader.viewmodel

import app.indelible.core.offline.reachableAgain
import app.indelible.reader.model.DataPanel
import app.indelible.reader.repository.ReaderRepository
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Deferred
import kotlinx.coroutines.Job
import kotlinx.coroutines.async
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.flow.emptyFlow
import kotlinx.coroutines.flow.filter
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.merge
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

internal const val RECONCILE_FIRST_RETRY_MS = 30_000L
internal const val RECONCILE_SECOND_RETRY_MS = 120_000L
private val RETRY_DELAYS_MS = listOf(RECONCILE_FIRST_RETRY_MS, RECONCILE_SECOND_RETRY_MS)

/** What the open reader watches to re-read its annotations; empty flows mean it never does. */
data class ReaderTriggers(
    val online: Flow<Boolean> = emptyFlow(),
    val reachable: Flow<Boolean?> = emptyFlow(),
    val changes: Flow<String> = emptyFlow(),
)

/** The network back, a request through, or this document resolved in Storage; first samples are a baseline. */
internal fun readerTriggers(
    documentId: String,
    triggers: ReaderTriggers,
): Flow<Unit> =
    merge(
        reachableAgain(triggers.online, triggers.reachable),
        triggers.changes.filter { it == documentId }.map { },
    )

/**
 * Runs [refresh] on every trigger. A trigger during a refresh joins it rather than starting another; a refresh
 * that gets no answer is tried again after each delay in [RETRY_DELAYS_MS], and a new trigger drops that wait.
 */
internal fun CoroutineScope.reconcileOn(
    triggers: Flow<Unit>,
    refresh: suspend () -> Boolean,
): Job {
    var inFlight: Deferred<Boolean>? = null
    return launch {
        triggers.collectLatest {
            var attempt = 0
            while (true) {
                val current = inFlight?.takeIf { it.isActive } ?: async { refresh() }.also { inFlight = it }
                if (current.await() || attempt == RETRY_DELAYS_MS.size) return@collectLatest
                delay(RETRY_DELAYS_MS[attempt++])
            }
        }
    }
}

/** Re-reads the open document's highlights, note and tags from the server; the document and position stay. */
internal fun CoroutineScope.reconcileReader(
    documentId: String,
    repository: ReaderRepository,
    uiState: MutableStateFlow<ReaderUiState>,
    activePanel: StateFlow<DataPanel>,
    triggers: ReaderTriggers,
): Job =
    reconcileOn(readerTriggers(documentId, triggers)) {
        refreshAnnotations(documentId, repository, uiState, activePanel)
    }

/** Reads the note from the server as its editor opens; the editor stays closed to typing until the read ends. */
internal fun CoroutineScope.refreshNoteOnOpen(
    documentId: String,
    repository: ReaderRepository,
    uiState: MutableStateFlow<ReaderUiState>,
): Job? {
    val open = uiState.value as? ReaderUiState.Success ?: return null
    uiState.updateIf({ true }) { it.copy(noteRefreshing = true) }
    return launch {
        try {
            repository.getItemNote(documentId, fromServer = true).onSuccess { fetched ->
                uiState.updateIf({ it.itemNote == open.itemNote }) {
                    it.copy(itemNote = fetched, itemNoteLoaded = true)
                }
            }
        } finally {
            uiState.updateIf({ true }) { it.copy(noteRefreshing = false) }
        }
    }
}

private suspend fun refreshAnnotations(
    documentId: String,
    repository: ReaderRepository,
    uiState: MutableStateFlow<ReaderUiState>,
    activePanel: StateFlow<DataPanel>,
): Boolean {
    val open = uiState.value as? ReaderUiState.Success ?: return true

    // A note that never loaded has no draft to protect, so the sheet being open does not hold it back.
    fun noteSheetOpen() = activePanel.value == DataPanel.NOTE

    return coroutineScope {
        val highlights =
            async {
                repository.listHighlights(documentId, fromServer = true).onSuccess { fetched ->
                    uiState.updateIf({ it.highlights === open.highlights }) {
                        it.copy(highlights = fetched, highlightsLoaded = true)
                    }
                }
            }
        val note =
            async {
                // The copy's stamp is the next edit's base, so it moves only together with what the open sheet shows.
                if (open.itemNoteLoaded && noteSheetOpen()) return@async Result.success(open.itemNote)
                repository.getItemNote(documentId, fromServer = true).onSuccess { fetched ->
                    uiState.updateIf({ !it.itemNoteLoaded || it.itemNote == open.itemNote && !noteSheetOpen() }) {
                        it.copy(itemNote = fetched, itemNoteLoaded = true)
                    }
                }
            }
        val tags =
            async {
                val entryId = open.item.libraryEntryId ?: return@async true
                repository
                    .getItemTags(entryId)
                    .onSuccess { fetched ->
                        uiState.updateIf({ it.itemTags === open.itemTags }) { it.copy(itemTags = fetched) }
                    }.isSuccess
            }
        highlights.await().isSuccess && note.await().isSuccess && tags.await()
    }
}

// A part is applied only over the value the fetch started from: a write made meanwhile wins.
private inline fun MutableStateFlow<ReaderUiState>.updateIf(
    unchanged: (ReaderUiState.Success) -> Boolean,
    transform: (ReaderUiState.Success) -> ReaderUiState.Success,
) {
    update { state -> if (state is ReaderUiState.Success && unchanged(state)) transform(state) else state }
}
