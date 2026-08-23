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
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.library_empty_archive_body
import indelible.composeapp.generated.resources.library_empty_archive_caption
import indelible.composeapp.generated.resources.library_empty_archive_kicker
import indelible.composeapp.generated.resources.library_empty_archive_title
import indelible.composeapp.generated.resources.library_empty_inbox_body
import indelible.composeapp.generated.resources.library_empty_inbox_caption
import indelible.composeapp.generated.resources.library_empty_inbox_kicker
import indelible.composeapp.generated.resources.library_empty_inbox_title
import indelible.composeapp.generated.resources.library_empty_later_body
import indelible.composeapp.generated.resources.library_empty_later_caption
import indelible.composeapp.generated.resources.library_empty_later_kicker
import indelible.composeapp.generated.resources.library_empty_later_title
import org.jetbrains.compose.resources.StringResource
import org.jetbrains.compose.resources.stringResource

private data class EmptyLibraryCopy(
    val kicker: StringResource,
    val title: StringResource,
    val body: StringResource,
    val caption: StringResource,
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
                    kicker = Res.string.library_empty_inbox_kicker,
                    title = Res.string.library_empty_inbox_title,
                    body = Res.string.library_empty_inbox_body,
                    caption = Res.string.library_empty_inbox_caption,
                )
            TriageFilter.LATER ->
                EmptyLibraryCopy(
                    kicker = Res.string.library_empty_later_kicker,
                    title = Res.string.library_empty_later_title,
                    body = Res.string.library_empty_later_body,
                    caption = Res.string.library_empty_later_caption,
                )
            TriageFilter.ARCHIVE ->
                EmptyLibraryCopy(
                    kicker = Res.string.library_empty_archive_kicker,
                    title = Res.string.library_empty_archive_title,
                    body = Res.string.library_empty_archive_body,
                    caption = Res.string.library_empty_archive_caption,
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
                    text = stringResource(copy.kicker),
                    style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
                Spacer(modifier = Modifier.height(IndelibleSpacing.step4))
                Text(
                    text = stringResource(copy.title),
                    style = MaterialTheme.typography.titleMedium,
                    color = MaterialTheme.colorScheme.onSurface,
                )
                Spacer(modifier = Modifier.height(IndelibleSpacing.step6))
                Text(
                    text = stringResource(copy.body),
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
            text = stringResource(copy.caption),
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
