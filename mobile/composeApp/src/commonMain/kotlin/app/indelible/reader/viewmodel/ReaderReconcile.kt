package app.indelible.reader.viewmodel

import app.indelible.library.ui.reachableAgain
import app.indelible.reader.repository.ReaderRepository
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
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
 * Runs [refresh] on every trigger. A refresh that gets no answer is tried again after each delay in
 * [RETRY_DELAYS_MS], then waits for the next trigger; a new trigger drops a pending retry.
 */
internal fun CoroutineScope.reconcileOn(
    triggers: Flow<Unit>,
    refresh: suspend () -> Boolean,
): Job =
    launch {
        triggers.collectLatest {
            var attempt = 0
            while (!refresh() && attempt < RETRY_DELAYS_MS.size) {
                delay(RETRY_DELAYS_MS[attempt++])
            }
        }
    }

/** Re-reads the open document's highlights, note and tags; the document and the position stay. */
internal fun CoroutineScope.reconcileReader(
    documentId: String,
    repository: ReaderRepository,
    uiState: MutableStateFlow<ReaderUiState>,
    triggers: ReaderTriggers,
): Job =
    reconcileOn(readerTriggers(documentId, triggers)) {
        refreshAnnotations(documentId, repository, uiState)
    }

private suspend fun refreshAnnotations(
    documentId: String,
    repository: ReaderRepository,
    uiState: MutableStateFlow<ReaderUiState>,
): Boolean {
    val open = uiState.value as? ReaderUiState.Success ?: return true
    val highlights = repository.listHighlights(documentId)
    val note = repository.getItemNote(documentId)
    val tags = open.item.libraryEntryId?.let { repository.getItemTags(it) }
    uiState.update { state ->
        if (state !is ReaderUiState.Success) return@update state
        state.copy(
            highlights = highlights.getOrDefault(state.highlights),
            highlightsLoaded = state.highlightsLoaded || highlights.isSuccess,
            itemNote = note.getOrDefault(state.itemNote),
            itemNoteLoaded = state.itemNoteLoaded || note.isSuccess,
            itemTags = tags?.getOrDefault(state.itemTags) ?: state.itemTags,
        )
    }
    return highlights.isSuccess && note.isSuccess
}
