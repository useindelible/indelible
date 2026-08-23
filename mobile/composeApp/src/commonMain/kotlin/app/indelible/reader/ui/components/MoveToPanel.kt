package app.indelible.reader.ui.components

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Check
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.tooling.preview.Preview
import app.indelible.ui.theme.AppTheme
import app.indelible.ui.theme.IndelibleIcons
import app.indelible.ui.theme.IndelibleShape
import app.indelible.ui.theme.IndelibleSpacing
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.reader_move_archive
import indelible.composeapp.generated.resources.reader_move_current_cd
import indelible.composeapp.generated.resources.reader_move_inbox
import indelible.composeapp.generated.resources.reader_move_later
import org.jetbrains.compose.resources.StringResource
import org.jetbrains.compose.resources.stringResource

/**
 * Move panel: relocates the article between the three triage buckets. The row
 * matching the item's current [currentState] carries a check; tapping any row
 * requests a move to that bucket. State strings mirror the API's triage values.
 */
@Composable
fun MoveToPanel(
    currentState: String?,
    onMove: (String) -> Unit,
    modifier: Modifier = Modifier,
) {
    val current = currentState?.lowercase()
    Column(modifier = modifier.fillMaxWidth()) {
        moveTargets.forEach { target ->
            MoveRow(
                icon = target.icon,
                label = stringResource(target.labelRes),
                selected = current == target.apiValue,
                onClick = { onMove(target.apiValue) },
            )
        }
    }
}

private data class MoveTarget(
    val apiValue: String,
    val labelRes: StringResource,
    val icon: ImageVector,
)

private val moveTargets: List<MoveTarget>
    get() =
        listOf(
            MoveTarget("inbox", Res.string.reader_move_inbox, IndelibleIcons.Inbox),
            MoveTarget("later", Res.string.reader_move_later, IndelibleIcons.Clock),
            MoveTarget("archive", Res.string.reader_move_archive, IndelibleIcons.Archive),
        )

@Composable
private fun MoveRow(
    icon: ImageVector,
    label: String,
    selected: Boolean,
    onClick: () -> Unit,
) {
    val accent = MaterialTheme.colorScheme.primary
    Row(
        modifier =
            Modifier
                .fillMaxWidth()
                .clip(IndelibleShape.lg)
                .clickable(onClickLabel = label, onClick = onClick)
                .padding(IndelibleSpacing.step12),
        horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step12),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Icon(
            imageVector = icon,
            contentDescription = null,
            tint = if (selected) accent else MaterialTheme.colorScheme.onSurfaceVariant,
            modifier = Modifier.size(IndelibleSpacing.step24),
        )
        Text(
            text = label,
            style = MaterialTheme.typography.bodyLarge,
            color = if (selected) accent else MaterialTheme.colorScheme.onSurface,
            modifier = Modifier.weight(1f),
        )
        if (selected) {
            Icon(
                imageVector = Icons.Filled.Check,
                contentDescription = stringResource(Res.string.reader_move_current_cd),
                tint = accent,
                modifier = Modifier.size(IndelibleSpacing.step20),
            )
        }
    }
}

@Preview
@Composable
private fun MoveToPanelPreviewLight() {
    AppTheme(darkTheme = false) {
        Surface {
            MoveToPanel(
                currentState = "inbox",
                onMove = {},
                modifier = Modifier.padding(IndelibleSpacing.screenPaddingH),
            )
        }
    }
}

@Preview
@Composable
private fun MoveToPanelPreviewDark() {
    AppTheme(darkTheme = true) {
        Surface {
            MoveToPanel(
                currentState = "archive",
                onMove = {},
                modifier = Modifier.padding(IndelibleSpacing.screenPaddingH),
            )
        }
    }
}
