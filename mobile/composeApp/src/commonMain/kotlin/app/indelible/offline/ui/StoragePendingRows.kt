package app.indelible.offline.ui

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextOverflow
import app.indelible.core.offline.OutboxKind
import app.indelible.offline.viewmodel.ChangeStatus
import app.indelible.offline.viewmodel.PendingChange
import app.indelible.profile.ui.components.SettingsSection
import app.indelible.ui.components.IndelibleButton
import app.indelible.ui.components.IndelibleButtonStyle
import app.indelible.ui.theme.IndelibleSpacing
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.common_retry
import indelible.composeapp.generated.resources.offline_all_synced
import indelible.composeapp.generated.resources.offline_change_highlight_color
import indelible.composeapp.generated.resources.offline_change_highlight_create
import indelible.composeapp.generated.resources.offline_change_highlight_delete
import indelible.composeapp.generated.resources.offline_change_highlight_note
import indelible.composeapp.generated.resources.offline_change_highlight_tags
import indelible.composeapp.generated.resources.offline_change_note
import indelible.composeapp.generated.resources.offline_change_progress
import indelible.composeapp.generated.resources.offline_pending
import indelible.composeapp.generated.resources.offline_status_blocked
import indelible.composeapp.generated.resources.offline_status_failed
import indelible.composeapp.generated.resources.offline_status_failed_reason
import indelible.composeapp.generated.resources.offline_status_pending
import indelible.composeapp.generated.resources.offline_status_retrying
import indelible.composeapp.generated.resources.offline_status_superseded
import org.jetbrains.compose.resources.StringResource
import org.jetbrains.compose.resources.stringResource

internal fun LazyListScope.pendingSection(
    changes: List<PendingChange>,
    onRetry: (String) -> Unit,
) {
    item { SettingsSection(title = stringResource(Res.string.offline_pending)) {} }
    if (changes.isEmpty()) {
        item { StorageNote(stringResource(Res.string.offline_all_synced)) }
        return
    }
    items(changes, key = { "change:${it.id}" }) { change ->
        Column {
            ChangeRow(change, onRetry)
            change.blocked.forEach { ChangeRow(it, onRetry, nested = true) }
        }
    }
}

@Composable
private fun ChangeRow(
    change: PendingChange,
    onRetry: (String) -> Unit,
    nested: Boolean = false,
) {
    val start = if (nested) IndelibleSpacing.rowPaddingH + IndelibleSpacing.step16 else IndelibleSpacing.rowPaddingH
    Row(
        modifier =
            Modifier
                .fillMaxWidth()
                .padding(start = start, end = IndelibleSpacing.step8),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Column(
            modifier =
                Modifier
                    .weight(1f)
                    .padding(vertical = IndelibleSpacing.step8),
        ) {
            Text(
                text = stringResource(change.kind.label()),
                style = MaterialTheme.typography.titleSmall,
            )
            change.documentTitle?.let {
                Text(
                    text = it,
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
            }
            Text(
                text = statusText(change),
                style = MaterialTheme.typography.bodySmall,
                color =
                    if (change.status == ChangeStatus.FAILED) {
                        MaterialTheme.colorScheme.error
                    } else {
                        MaterialTheme.colorScheme.onSurfaceVariant
                    },
            )
        }
        if (change.canRetry) {
            IndelibleButton(
                text = stringResource(Res.string.common_retry),
                onClick = { onRetry(change.id) },
                style = IndelibleButtonStyle.Text,
                compact = true,
            )
        }
    }
}

@Composable
private fun statusText(change: PendingChange): String =
    when (change.status) {
        ChangeStatus.PENDING -> stringResource(Res.string.offline_status_pending)
        ChangeStatus.RETRYING -> stringResource(Res.string.offline_status_retrying)
        ChangeStatus.BLOCKED -> stringResource(Res.string.offline_status_blocked)
        ChangeStatus.SUPERSEDED -> stringResource(Res.string.offline_status_superseded)
        ChangeStatus.FAILED ->
            change.error?.let { stringResource(Res.string.offline_status_failed_reason, it) }
                ?: stringResource(Res.string.offline_status_failed)
    }

private fun OutboxKind.label(): StringResource =
    when (this) {
        OutboxKind.READING_EVENT -> Res.string.offline_change_progress
        OutboxKind.HIGHLIGHT_CREATE -> Res.string.offline_change_highlight_create
        OutboxKind.HIGHLIGHT_COLOR -> Res.string.offline_change_highlight_color
        OutboxKind.HIGHLIGHT_NOTE -> Res.string.offline_change_highlight_note
        OutboxKind.HIGHLIGHT_TAGS -> Res.string.offline_change_highlight_tags
        OutboxKind.HIGHLIGHT_DELETE -> Res.string.offline_change_highlight_delete
        OutboxKind.DOCUMENT_NOTE -> Res.string.offline_change_note
    }
