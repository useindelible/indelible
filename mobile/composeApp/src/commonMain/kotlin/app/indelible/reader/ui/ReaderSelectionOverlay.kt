package app.indelible.reader.ui

import androidx.compose.foundation.layout.BoxScope
import androidx.compose.foundation.layout.offset
import androidx.compose.material3.SnackbarHostState
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.ClipboardManager
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.dp
import app.indelible.reader.model.HighlightColor
import app.indelible.reader.model.TagData
import app.indelible.reader.ui.components.HighlightToolbar
import app.indelible.reader.viewmodel.ReaderViewModel
import app.indelible.ui.theme.IndelibleSpacing
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.reader_copied_clipboard
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch
import org.jetbrains.compose.resources.stringResource

@Composable
internal fun BoxScope.HighlightToolbarOverlay(
    sel: SelectedTextInfo,
    defaultHighlightColor: HighlightColor,
    snackbarHostState: SnackbarHostState,
    coroutineScope: CoroutineScope,
    clipboardManager: ClipboardManager,
    viewModel: ReaderViewModel,
    onSelectedTextChanged: (SelectedTextInfo?) -> Unit,
    onAvailableTagsChanged: (List<TagData>) -> Unit,
    onTagSheetHighlightIdChanged: (String?) -> Unit,
) {
    val copiedMessage = stringResource(Res.string.reader_copied_clipboard)
    // sel.rect.y is a CSS viewport-relative pixel from getBoundingClientRect().
    // CSS pixels from a standard-viewport WebView equal dp, so use .dp
    // directly — .toDp() would incorrectly divide by display density.
    val yOffset = sel.rect.y.dp - IndelibleSpacing.step64
    HighlightToolbar(
        onColorSelected = { color ->
            viewModel.createHighlight(
                color = color,
                textContent = sel.text,
                startOffset = sel.startOffset.toLong(),
                endOffset = sel.endOffset.toLong(),
            )
            onSelectedTextChanged(null)
        },
        onTagSelected = {
            val selCopy = sel
            onSelectedTextChanged(null)
            viewModel.createHighlightForTag(
                color = defaultHighlightColor,
                textContent = selCopy.text,
                startOffset = selCopy.startOffset.toLong(),
                endOffset = selCopy.endOffset.toLong(),
            ) { highlightId ->
                viewModel.loadTagsForPicker { tags ->
                    onAvailableTagsChanged(tags)
                    onTagSheetHighlightIdChanged(highlightId)
                }
            }
        },
        onNoteSelected = {
            viewModel.createHighlight(
                color = defaultHighlightColor,
                textContent = sel.text,
                startOffset = sel.startOffset.toLong(),
                endOffset = sel.endOffset.toLong(),
            )
            onSelectedTextChanged(null)
        },
        onCopySelected = {
            clipboardManager.setText(AnnotatedString(sel.text))
            onSelectedTextChanged(null)
            coroutineScope.launch {
                snackbarHostState.showSnackbar(copiedMessage)
            }
        },
        modifier =
            Modifier
                .align(Alignment.TopCenter)
                .offset {
                    IntOffset(0, yOffset.roundToPx().coerceAtLeast(0))
                },
    )
}
