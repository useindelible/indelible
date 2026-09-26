package app.indelible.offline.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.em
import app.indelible.core.i18n.relativeTimeText
import app.indelible.core.offline.OutboxKind
import app.indelible.offline.viewmodel.ChangeStatus
import app.indelible.offline.viewmodel.PendingChange
import app.indelible.ui.theme.IndelibleSpacing
import app.indelible.ui.theme.IndelibleTheme
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.common_retry
import indelible.composeapp.generated.resources.offline_change_highlight_color
import indelible.composeapp.generated.resources.offline_change_highlight_create
import indelible.composeapp.generated.resources.offline_change_highlight_delete
import indelible.composeapp.generated.resources.offline_change_highlight_note
import indelible.composeapp.generated.resources.offline_change_highlight_tags
import indelible.composeapp.generated.resources.offline_change_note
import indelible.composeapp.generated.resources.offline_change_progress
import indelible.composeapp.generated.resources.offline_pending
import indelible.composeapp.generated.resources.offline_pending_changes
import indelible.composeapp.generated.resources.offline_pending_footnote
import indelible.composeapp.generated.resources.offline_queue_failed
import indelible.composeapp.generated.resources.offline_status_blocked
import indelible.composeapp.generated.resources.offline_status_failed_reason
import indelible.composeapp.generated.resources.offline_status_pending
import indelible.composeapp.generated.resources.offline_status_retrying
import indelible.composeapp.generated.resources.offline_status_superseded
import kotlinx.datetime.Clock
import kotlinx.datetime.Instant
import org.jetbrains.compose.resources.StringResource
import org.jetbrains.compose.resources.pluralStringResource
import org.jetbrains.compose.resources.stringResource

internal fun LazyListScope.pendingSection(
    changes: List<PendingChange>,
    onRetry: (String) -> Unit,
) {
    if (changes.isEmpty()) return
    item {
        SectionHead(
            label = stringResource(Res.string.offline_pending),
            count = pluralStringResource(Res.plurals.offline_pending_changes, changes.size, changes.size),
            modifier =
                Modifier.padding(
                    start = IndelibleSpacing.rowPaddingH,
                    end = IndelibleSpacing.rowPaddingH,
                    top = SECTION_SPACING,
                    bottom = IndelibleSpacing.step12,
                ),
        )
    }
    val rows = changes.flatMap { change -> listOf(change to false) + change.blocked.map { it to true } }
    itemsIndexed(rows, key = { _, (change, _) -> "change:${change.id}" }) { index, (change, nested) ->
        ChangeRow(change, nested, first = index == 0, last = index == rows.lastIndex, onRetry = onRetry)
    }
    item { StorageNote(stringResource(Res.string.offline_pending_footnote)) }
}

@Composable
private fun ChangeRow(
    change: PendingChange,
    nested: Boolean,
    first: Boolean,
    last: Boolean,
    onRetry: (String) -> Unit,
) {
    val colors = MaterialTheme.colorScheme
    Column(
        modifier =
            Modifier
                .padding(horizontal = IndelibleSpacing.rowPaddingH)
                .fillMaxWidth()
                .cardCell(first, last, colors.surfaceVariant, colors.outlineVariant)
                .padding(
                    start = if (nested) IndelibleSpacing.step32 else IndelibleSpacing.step14,
                    end = IndelibleSpacing.step14,
                    top = IndelibleSpacing.step12,
                    bottom = IndelibleSpacing.step12,
                ),
    ) {
        Text(
            text = changeTitle(change),
            style = MaterialTheme.typography.titleSmall,
            color = colors.onSurface,
            maxLines = 2,
            overflow = TextOverflow.Ellipsis,
        )
        Text(
            text = changeMeta(change),
            style = storageMono(TRACK_META, FontWeight.Normal),
            color = IndelibleTheme.colors.textTertiary,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
            modifier = Modifier.padding(top = IndelibleSpacing.step4),
        )
        StatusLine(change)
        if (change.canRetry) {
            Box(modifier = Modifier.padding(top = IndelibleSpacing.step10)) {
                RowAction(
                    text = stringResource(Res.string.common_retry),
                    onClick = { onRetry(change.id) },
                    accent = true,
                )
            }
        }
    }
}

@Composable
private fun StatusLine(change: PendingChange) {
    val ink =
        when (change.status) {
            ChangeStatus.FAILED -> MaterialTheme.colorScheme.error
            ChangeStatus.RETRYING -> IndelibleTheme.colors.warning
            else -> IndelibleTheme.colors.textTertiary
        }
    Row(
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step6),
        modifier = Modifier.padding(top = IndelibleSpacing.step8),
    ) {
        Box(modifier = Modifier.size(STATUS_DOT).clip(CircleShape).background(ink))
        Text(text = statusText(change), style = storageMono(TRACK_STATUS), color = ink)
    }
}

/** The kind, then the words the change carries, the way the queue reads them back to the user. */
@Composable
private fun changeTitle(change: PendingChange): String {
    val kind = stringResource(change.kind.label())
    val detail = change.detail?.trim()?.takeIf { it.isNotEmpty() } ?: return kind
    val quoted = if (detail.length > DETAIL_CHARS) detail.take(DETAIL_CHARS).trimEnd() + "\u2026" else detail
    return "$kind \u00b7 \u201c$quoted\u201d"
}

@Composable
private fun changeMeta(change: PendingChange): String {
    val made = relativeTimeText(Instant.fromEpochMilliseconds(change.createdAt), Clock.System.now())
    return change.documentTitle?.let { "$it \u00b7 $made" } ?: made
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
                ?: stringResource(Res.string.offline_queue_failed)
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

private const val DETAIL_CHARS = 60
private val STATUS_DOT = 6.dp
private val TRACK_STATUS = 0.1.em
