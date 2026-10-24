package app.indelible.reader.viewmodel

import app.indelible.core.i18n.UiMessage
import app.indelible.reader.model.HighlightColor
import app.indelible.reader.model.TagData
import app.indelible.reader.repository.ReaderRepository
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.reader_error_create_highlight
import indelible.composeapp.generated.resources.reader_error_delete_highlight
import indelible.composeapp.generated.resources.reader_error_delete_note
import indelible.composeapp.generated.resources.reader_error_save_note
import indelible.composeapp.generated.resources.reader_error_save_tags
import indelible.composeapp.generated.resources.reader_error_update_highlight_color
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.launch

/** Highlight mutations for [ReaderViewModel], over the same state and effect streams. */
internal class ReaderHighlightActions(
    private val documentId: String,
    private val repository: ReaderRepository,
    private val scope: CoroutineScope,
    private val uiState: MutableStateFlow<ReaderUiState>,
    private val effects: MutableSharedFlow<ReaderEffect>,
) {
    fun createHighlight(
        color: HighlightColor,
        textContent: String,
        startOffset: Long,
        endOffset: Long,
        onCreated: (String) -> Unit = {},
    ) {
        scope.launch {
            repository
                .createHighlight(
                    itemId = documentId,
                    color = color.apiValue,
                    textContent = textContent,
                    startOffset = startOffset,
                    endOffset = endOffset,
                ).onSuccess { highlight ->
                    updateSuccessState { state ->
                        state.copy(highlights = state.highlights + highlight)
                    }
                    onCreated(highlight.id)
                }.onFailure {
                    effects.emit(ReaderEffect.ShowSnackbar(UiMessage(Res.string.reader_error_create_highlight)))
                }
        }
    }

    fun deleteHighlight(highlightId: String) {
        val state = (uiState.value as? ReaderUiState.Success) ?: return
        val removedHighlight = state.highlights.find { it.id == highlightId }
        updateSuccessState { s ->
            s.copy(highlights = s.highlights.filter { it.id != highlightId })
        }
        scope.launch {
            repository
                .deleteHighlight(documentId, highlightId)
                .onFailure {
                    if (removedHighlight != null) {
                        updateSuccessState { s ->
                            s.copy(highlights = s.highlights + removedHighlight)
                        }
                    }
                    effects.emit(ReaderEffect.ShowSnackbar(UiMessage(Res.string.reader_error_delete_highlight)))
                }
        }
    }

    fun updateHighlightColor(
        highlightId: String,
        color: HighlightColor,
    ) {
        scope.launch {
            repository
                .updateHighlightColor(documentId, highlightId, color.apiValue)
                .onSuccess { updated ->
                    updateSuccessState { state ->
                        state.copy(
                            highlights =
                                state.highlights.map { h ->
                                    // Only the colour was rewritten; the local entry keeps the rest.
                                    if (h.id == highlightId) h.copy(color = updated.color) else h
                                },
                        )
                    }
                }.onFailure {
                    effects.emit(
                        ReaderEffect.ShowSnackbar(UiMessage(Res.string.reader_error_update_highlight_color)),
                    )
                }
        }
    }

    fun upsertHighlightNote(
        highlightId: String,
        body: String,
    ) {
        scope.launch {
            repository
                .upsertHighlightNote(documentId, highlightId, body)
                .onSuccess { note ->
                    updateSuccessState { state ->
                        state.copy(
                            highlights =
                                state.highlights.map { h ->
                                    if (h.id == highlightId) h.copy(note = note) else h
                                },
                        )
                    }
                }.onFailure {
                    effects.emit(ReaderEffect.ShowSnackbar(UiMessage(Res.string.reader_error_save_note)))
                }
        }
    }

    fun deleteHighlightNote(highlightId: String) {
        scope.launch {
            repository
                .deleteHighlightNote(documentId, highlightId)
                .onSuccess {
                    updateSuccessState { state ->
                        state.copy(
                            highlights =
                                state.highlights.map { h ->
                                    if (h.id == highlightId) h.copy(note = null) else h
                                },
                        )
                    }
                }.onFailure {
                    effects.emit(ReaderEffect.ShowSnackbar(UiMessage(Res.string.reader_error_delete_note)))
                }
        }
    }

    fun setHighlightTags(
        highlightId: String,
        tags: List<String>,
    ) {
        updateSuccessState { state ->
            state.copy(
                highlights =
                    state.highlights.map { h ->
                        if (h.id == highlightId) h.copy(tags = tags) else h
                    },
            )
        }
        scope.launch {
            repository
                .setHighlightTags(documentId, highlightId, tags)
                .onFailure {
                    effects.emit(ReaderEffect.ShowSnackbar(UiMessage(Res.string.reader_error_save_tags)))
                }
        }
    }

    fun loadTagsForPicker(onResult: (List<TagData>) -> Unit) {
        scope.launch {
            repository
                .listTags()
                .onSuccess { tags -> onResult(tags) }
                .onFailure { onResult(emptyList()) }
        }
    }

    private fun updateSuccessState(transform: (ReaderUiState.Success) -> ReaderUiState.Success) {
        val current = uiState.value as? ReaderUiState.Success ?: return
        uiState.value = transform(current)
    }
}
