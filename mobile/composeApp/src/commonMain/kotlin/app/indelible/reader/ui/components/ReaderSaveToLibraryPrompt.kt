package app.indelible.reader.ui.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import app.indelible.ui.components.IndelibleButton
import app.indelible.ui.theme.AppTheme
import app.indelible.ui.theme.IndelibleSpacing
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.reader_action_save_to_library
import org.jetbrains.compose.resources.stringResource

/**
 * Shown in library-gated reader surfaces (triage, item tags) for a feed document that has not
 * been saved yet. Saving creates the library entry that unlocks organizing the item; reading,
 * highlighting, tagging highlights, notes, and Mila already work without it.
 */
@Composable
fun ReaderSaveToLibraryPrompt(
    onSave: () -> Unit,
    message: String,
    modifier: Modifier = Modifier,
) {
    Column(
        modifier =
            modifier
                .fillMaxWidth()
                .padding(IndelibleSpacing.step24),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(IndelibleSpacing.step12),
    ) {
        Text(
            text = message,
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            textAlign = TextAlign.Center,
        )
        IndelibleButton(
            text = stringResource(Res.string.reader_action_save_to_library),
            onClick = onSave,
            compact = true,
        )
    }
}

@Preview
@Composable
private fun ReaderSaveToLibraryPromptPreview() {
    AppTheme(darkTheme = false) {
        Surface {
            ReaderSaveToLibraryPrompt(
                onSave = {},
                message = "Save this item to your library to organize it.",
                modifier = Modifier.fillMaxWidth(),
            )
        }
    }
}
