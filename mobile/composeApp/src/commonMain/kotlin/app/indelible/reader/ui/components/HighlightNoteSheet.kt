package app.indelible.reader.ui.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.Text
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import app.indelible.reader.model.HighlightData
import app.indelible.ui.components.IndelibleButton
import app.indelible.ui.components.IndelibleButtonStyle
import app.indelible.ui.components.IndelibleTextField
import app.indelible.ui.theme.AppTheme
import app.indelible.ui.theme.IndelibleSpacing
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.common_save
import indelible.composeapp.generated.resources.reader_highlight_note_delete
import indelible.composeapp.generated.resources.reader_highlight_note_label
import indelible.composeapp.generated.resources.reader_highlight_note_title
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
    val sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true)
    ModalBottomSheet(onDismissRequest = onDismiss, sheetState = sheetState, modifier = modifier) {
        HighlightNoteEditor(highlight, onSave, onDeleteNote)
    }
}

@Composable
internal fun HighlightNoteEditor(
    highlight: HighlightData,
    onSave: (String) -> Unit,
    onDeleteNote: () -> Unit,
    modifier: Modifier = Modifier,
) {
    // The sheet edits the note it opened with; a note that arrives from a sync meanwhile is never deleted by Save.
    val opened = rememberSaveable(highlight.id) { mutableStateOf(highlight.note?.body) }.value
    var body by rememberSaveable(highlight.id) { mutableStateOf(opened.orEmpty()) }
    val hasNote = opened != null
    val changed = body.trim() != opened.orEmpty().trim()
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
            onValueChange = { body = it },
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
    AppTheme(darkTheme = false) { HighlightNoteEditor(previewHighlight(), onSave = {}, onDeleteNote = {}) }
}

@Preview
@Composable
private fun HighlightNoteEditorDarkPreview() {
    AppTheme(darkTheme = true) { HighlightNoteEditor(previewHighlight(), onSave = {}, onDeleteNote = {}) }
}
