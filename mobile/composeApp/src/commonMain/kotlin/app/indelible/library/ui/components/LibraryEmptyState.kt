package app.indelible.library.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import app.indelible.library.viewmodel.TriageFilter
import app.indelible.ui.components.ZeroedGhostRows
import app.indelible.ui.components.dashedZeroBorder
import app.indelible.ui.theme.AppTheme
import app.indelible.ui.theme.IndelibleShape
import app.indelible.ui.theme.IndelibleSpacing

private data class EmptyLibraryCopy(
    val kicker: String,
    val title: String,
    val body: String,
    val caption: String,
)

@Composable
fun LibraryEmptyState(
    triageFilter: TriageFilter,
    modifier: Modifier = Modifier,
) {
    val copy =
        when (triageFilter) {
            TriageFilter.INBOX ->
                EmptyLibraryCopy(
                    kicker = "First save",
                    title = "Save a link and it appears right here",
                    body = "Use the share sheet, browser extension, or any connected app to add your first item.",
                    caption = "Saved items land in this list",
                )
            TriageFilter.LATER ->
                EmptyLibraryCopy(
                    kicker = "Nothing queued",
                    title = "Items saved for later appear right here",
                    body = "Move something from your inbox when you want it waiting for another day.",
                    caption = "Your later queue fills this list",
                )
            TriageFilter.ARCHIVE ->
                EmptyLibraryCopy(
                    kicker = "Nothing archived",
                    title = "Finished items stay available right here",
                    body = "Archive something after reading it and it remains easy to find.",
                    caption = "Archived items fill this list",
                )
        }

    Column(
        modifier =
            modifier
                .fillMaxSize()
                .padding(
                    horizontal = IndelibleSpacing.rowPaddingH,
                    vertical = IndelibleSpacing.step8,
                ),
    ) {
        Row(
            modifier =
                Modifier
                    .fillMaxWidth()
                    .dashedZeroBorder(MaterialTheme.colorScheme.outline)
                    .padding(IndelibleSpacing.step12),
            verticalAlignment = Alignment.Top,
        ) {
            Box(
                modifier =
                    Modifier
                        .size(IndelibleSpacing.step56)
                        .clip(IndelibleShape.lg)
                        .background(MaterialTheme.colorScheme.surfaceContainerHighest),
                contentAlignment = Alignment.Center,
            ) {
                Icon(
                    imageVector = Icons.Filled.Add,
                    contentDescription = null,
                    tint = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
            Spacer(modifier = Modifier.width(IndelibleSpacing.step14))
            Column(modifier = Modifier.weight(1f)) {
                Text(
                    text = copy.kicker,
                    style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
                Spacer(modifier = Modifier.height(IndelibleSpacing.step4))
                Text(
                    text = copy.title,
                    style = MaterialTheme.typography.titleMedium,
                    color = MaterialTheme.colorScheme.onSurface,
                )
                Spacer(modifier = Modifier.height(IndelibleSpacing.step6))
                Text(
                    text = copy.body,
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
        }
        Spacer(modifier = Modifier.height(IndelibleSpacing.step8))
        ZeroedGhostRows(
            borderColor = MaterialTheme.colorScheme.outline,
            lineColor = MaterialTheme.colorScheme.outlineVariant,
        )
        Spacer(modifier = Modifier.height(IndelibleSpacing.step12))
        Text(
            text = copy.caption,
            style = MaterialTheme.typography.labelMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            textAlign = TextAlign.Center,
            modifier = Modifier.fillMaxWidth(),
        )
    }
}

@Preview(showBackground = true)
@Composable
private fun LibraryEmptyStatePreviewLight() {
    AppTheme(darkTheme = false) {
        Surface {
            LibraryEmptyState(triageFilter = TriageFilter.INBOX)
        }
    }
}

@Preview(showBackground = true, uiMode = 0x20)
@Composable
private fun LibraryEmptyStatePreviewDark() {
    AppTheme(darkTheme = true) {
        Surface {
            LibraryEmptyState(triageFilter = TriageFilter.LATER)
        }
    }
}
