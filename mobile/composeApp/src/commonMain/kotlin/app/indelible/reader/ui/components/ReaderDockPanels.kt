package app.indelible.reader.ui.components

import androidx.compose.runtime.Composable
import app.indelible.reader.model.DataPanel
import app.indelible.reader.model.HighlightColor
import app.indelible.reader.model.ReaderPreferences
import app.indelible.reader.model.TagData
import app.indelible.reader.playback.PlaybackState
import app.indelible.reader.playback.ReaderVoice
import app.indelible.reader.viewmodel.ReaderUiState
import app.indelible.reader.viewmodel.TocStatus

/**
 * Bottom-anchored reader panels driven by the dock's [DataPanel] selection. Each
 * is a [ReaderBottomSheetScaffold] that shows when [activePanel] matches; only one
 * is ever visible. Kept out of [app.indelible.reader.ui.ReaderScreen] so that
 * screen stays within the file-size cap as more panels land in later phases.
 */
@Composable
fun ReaderDockPanels(
    activePanel: DataPanel,
    state: ReaderUiState.Success,
    defaultHighlightColor: HighlightColor,
    availableTags: List<TagData>,
    playbackState: PlaybackState,
    voices: List<ReaderVoice>,
    onDismiss: () -> Unit,
    onPreferencesChanged: (ReaderPreferences) -> Unit,
    onDefaultHighlightColorSelected: (HighlightColor) -> Unit,
    onSaveNote: (String) -> Unit,
    onTagsChanged: (List<String>) -> Unit,
    onEditNote: () -> Unit,
    onMove: (String) -> Unit,
    onSaveToLibrary: () -> Unit,
    onTogglePlay: () -> Unit,
    onSeek: (Long) -> Unit,
    onSkip: (Long) -> Unit,
    onSetSpeed: (Float) -> Unit,
    onSelectVoice: (String) -> Unit,
    onSetSleepTimer: (Int?) -> Unit,
    onShare: () -> Unit,
    onTocEntryTapped: (app.indelible.reader.model.ArticleTocEntry) -> Unit,
) {
    ReaderBottomSheetScaffold(
        visible = activePanel == DataPanel.AA,
        eyebrow = "Display",
        onDismiss = onDismiss,
    ) {
        DisplaySettingsPanel(
            preferences = state.preferences,
            onPreferencesChanged = onPreferencesChanged,
        )
    }

    ReaderBottomSheetScaffold(
        visible = activePanel == DataPanel.HL,
        eyebrow = "Highlight",
        onDismiss = onDismiss,
    ) {
        HighlightStylePanel(
            selectedColor = defaultHighlightColor,
            style = state.preferences.highlightStyle,
            onColorSelected = onDefaultHighlightColorSelected,
            onStyleSelected = { onPreferencesChanged(state.preferences.copy(highlightStyle = it)) },
        )
    }

    ReaderBottomSheetScaffold(
        visible = activePanel == DataPanel.NOTE,
        eyebrow = "Notes & Tags",
        onDismiss = onDismiss,
    ) {
        NoteTagsSwitcherSheet(
            note = state.itemNote.orEmpty(),
            tags = state.itemTags,
            availableTags = availableTags,
            onSaveNote = onSaveNote,
            onTagsChanged = onTagsChanged,
            tagsEnabled = state.item.saved,
            onSaveToLibrary = onSaveToLibrary,
        )
    }

    ReaderBottomSheetScaffold(
        visible = activePanel == DataPanel.MOVE,
        eyebrow = "Move",
        onDismiss = onDismiss,
    ) {
        if (state.item.saved) {
            MoveToPanel(
                currentState = state.item.triageState,
                onMove = onMove,
            )
        } else {
            ReaderSaveToLibraryPrompt(
                onSave = onSaveToLibrary,
                message = "Save this item to your library to move it between Inbox, Later, and Archive.",
            )
        }
    }

    ReaderBottomSheetScaffold(
        visible = activePanel == DataPanel.INFO,
        eyebrow = "Item details",
        onDismiss = onDismiss,
        fillHeightFraction = 0.82f,
    ) {
        ItemRecordPanel(
            item = state.item,
            note = state.itemNote,
            tags = state.itemTags,
            availableTags = availableTags,
            highlights = state.highlights,
            progress = state.progress,
            onEditNote = onEditNote,
            onTagsChanged = onTagsChanged,
            entities = state.entities,
            onSaveToLibrary = onSaveToLibrary,
            onShare = onShare,
        )
    }

    ReaderBottomSheetScaffold(
        visible = activePanel == DataPanel.CONTENTS,
        eyebrow = contentsEyebrow(state.toc.status, state.progress.toInt()),
        onDismiss = onDismiss,
        fillHeightFraction = if (state.toc.status == TocStatus.READY) 0.6f else null,
    ) {
        ContentsPanel(
            toc = state.toc,
            onEntryTap = onTocEntryTapped,
        )
    }

    ReaderBottomSheetScaffold(
        visible = activePanel == DataPanel.LISTEN,
        eyebrow = "Listen",
        onDismiss = onDismiss,
    ) {
        ListenPanel(
            title = state.item.title,
            source = state.item.domain ?: "Indelible",
            state = playbackState,
            voices = voices,
            onTogglePlay = onTogglePlay,
            onSeek = onSeek,
            onSkip = onSkip,
            onSetSpeed = onSetSpeed,
            onSelectVoice = onSelectVoice,
            onSetSleepTimer = onSetSleepTimer,
        )
    }
}
