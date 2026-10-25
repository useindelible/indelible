package app.indelible.reader.ui.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import app.indelible.reader.model.HighlightData
import app.indelible.reader.model.HighlightNoteData
import app.indelible.ui.components.IndelibleButton
import app.indelible.ui.components.IndelibleButtonStyle
import app.indelible.ui.components.IndelibleTextField
import app.indelible.ui.theme.AppTheme
import app.indelible.ui.theme.IndelibleSpacing
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.common_save
import indelible.composeapp.generated.resources.reader_highlight_note_delete
import indelible.composeapp.generated.resources.reader_highlight_note_discard
import indelible.composeapp.generated.resources.reader_highlight_note_discard_body
import indelible.composeapp.generated.resources.reader_highlight_note_discard_title
import indelible.composeapp.generated.resources.reader_highlight_note_keep
import indelible.composeapp.generated.resources.reader_highlight_note_label
import indelible.composeapp.generated.resources.reader_highlight_note_title
import kotlinx.coroutines.launch
import org.jetbrains.compose.resources.stringResource

private const val NOTE_LINES = 4
private const val NOTE_MAX_LINES = 8
private const val QUOTE_LINES = 3

/** One highlight's note in every reader: Save writes the trimmed body; an empty body or Delete note removes it. */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun HighlightNoteSheet(
    highlight: HighlightData,
    onSave: (String) -> Unit,
    onDeleteNote: () -> Unit,
    onDismiss: () -> Unit,
    modifier: Modifier = Modifier,
) {
    // The sheet edits the note it opened with; a note that arrives from a sync meanwhile is never deleted by Save.
    val opened = rememberSaveable(highlight.id) { mutableStateOf(highlight.note?.body) }.value
    var body by rememberSaveable(highlight.id) { mutableStateOf(opened.orEmpty()) }
    var askDiscard by rememberSaveable { mutableStateOf(false) }
    val changed = body.trim() != opened.orEmpty().trim()
    val sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true)
    val scope = rememberCoroutineScope()
    ModalBottomSheet(
        onDismissRequest = { if (changed) askDiscard = true else onDismiss() },
        sheetState = sheetState,
        modifier = modifier,
    ) {
        HighlightNoteEditor(highlight, body, { body = it }, opened != null, changed, onSave, onDeleteNote)
    }
    if (askDiscard) {
        DiscardNoteDialog(
            onDiscard = {
                askDiscard = false
                onDismiss()
            },
            onKeep = {
                askDiscard = false
                // The sheet has already slid away before it asks to be dismissed.
                scope.launch { sheetState.show() }
            },
        )
    }
}

@Composable
private fun DiscardNoteDialog(
    onDiscard: () -> Unit,
    onKeep: () -> Unit,
) {
    AlertDialog(
        onDismissRequest = onKeep,
        title = { Text(stringResource(Res.string.reader_highlight_note_discard_title)) },
        text = {
            Text(
                text = stringResource(Res.string.reader_highlight_note_discard_body),
                style = MaterialTheme.typography.bodyMedium,
            )
        },
        confirmButton = {
            TextButton(onClick = onDiscard) {
                Text(
                    text = stringResource(Res.string.reader_highlight_note_discard),
                    color = MaterialTheme.colorScheme.error,
                )
            }
        },
        dismissButton = {
            TextButton(onClick = onKeep) {
                Text(stringResource(Res.string.reader_highlight_note_keep))
            }
        },
    )
}

@Composable
internal fun HighlightNoteEditor(
    highlight: HighlightData,
    body: String,
    onBodyChange: (String) -> Unit,
    hasNote: Boolean,
    changed: Boolean,
    onSave: (String) -> Unit,
    onDeleteNote: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val save = {
        val trimmed = body.trim()
        if (trimmed.isEmpty()) {
            if (hasNote) onDeleteNote()
        } else {
            onSave(trimmed)
        }
    }
    Column(
        modifier =
            modifier
                .fillMaxWidth()
                .padding(horizontal = IndelibleSpacing.screenPaddingH, vertical = IndelibleSpacing.step16),
    ) {
        Text(
            text = stringResource(Res.string.reader_highlight_note_title),
            style = MaterialTheme.typography.titleLarge,
            color = MaterialTheme.colorScheme.onSurface,
        )
        Spacer(Modifier.height(IndelibleSpacing.step8))
        Text(
            text = highlight.textContent,
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            maxLines = QUOTE_LINES,
            overflow = TextOverflow.Ellipsis,
        )
        Spacer(Modifier.height(IndelibleSpacing.step16))
        IndelibleTextField(
            value = body,
            onValueChange = onBodyChange,
            label = stringResource(Res.string.reader_highlight_note_label),
            singleLine = false,
            minLines = NOTE_LINES,
            maxLines = NOTE_MAX_LINES,
            imeAction = ImeAction.Default,
            modifier = Modifier.fillMaxWidth(),
        )
        Spacer(Modifier.height(IndelibleSpacing.step16))
        Row(horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step12), modifier = Modifier.fillMaxWidth()) {
            if (hasNote) {
                IndelibleButton(
                    text = stringResource(Res.string.reader_highlight_note_delete),
                    onClick = onDeleteNote,
                    style = IndelibleButtonStyle.OutlinedDestructive,
                    modifier = Modifier.weight(1f),
                )
            }
            IndelibleButton(
                text = stringResource(Res.string.common_save),
                onClick = save,
                enabled = changed,
                modifier = Modifier.weight(1f),
            )
        }
    }
}

@Preview
@Composable
private fun HighlightNoteEditorLightPreview() {
    AppTheme(darkTheme = false) {
        Surface { HighlightNoteEditor(previewHighlight(), "", {}, false, false, {}, {}) }
    }
}

@Preview
@Composable
private fun HighlightNoteEditorDarkPreview() {
    val base = previewHighlight()
    val highlight =
        base.copy(
            note =
                HighlightNoteData(
                    id = "hnt_preview",
                    highlightId = base.id,
                    body = "Remember this",
                    createdAt = base.createdAt,
                    updatedAt = base.updatedAt,
                ),
        )
    AppTheme(darkTheme = true) {
        Surface { HighlightNoteEditor(highlight, "Remember this", {}, true, false, {}, {}) }
    }
}
