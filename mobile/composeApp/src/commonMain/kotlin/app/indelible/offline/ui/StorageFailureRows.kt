package app.indelible.offline.ui

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import app.indelible.core.offline.SyncFailure
import app.indelible.offline.viewmodel.PendingChange
import app.indelible.ui.theme.IndelibleSpacing
import app.indelible.ui.theme.IndelibleTheme
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.common_retry
import indelible.composeapp.generated.resources.offline_action_copy_text
import indelible.composeapp.generated.resources.offline_action_delete_permanently
import indelible.composeapp.generated.resources.offline_action_discard
import indelible.composeapp.generated.resources.offline_action_keep_it
import indelible.composeapp.generated.resources.offline_action_retry_now
import indelible.composeapp.generated.resources.offline_action_view_both
import indelible.composeapp.generated.resources.offline_discard_body_alone
import indelible.composeapp.generated.resources.offline_discard_body_dependants
import indelible.composeapp.generated.resources.offline_discard_title
import indelible.composeapp.generated.resources.offline_failure_anchor_title
import indelible.composeapp.generated.resources.offline_failure_conflict_title
import indelible.composeapp.generated.resources.offline_failure_deleted_title
import indelible.composeapp.generated.resources.offline_failure_rejected_title
import indelible.composeapp.generated.resources.offline_failure_too_old_line
import indelible.composeapp.generated.resources.offline_failure_too_old_title
import indelible.composeapp.generated.resources.offline_feature_highlights
import indelible.composeapp.generated.resources.offline_feature_notes
import indelible.composeapp.generated.resources.offline_feature_reading_events
import indelible.composeapp.generated.resources.offline_queue_failed
import indelible.composeapp.generated.resources.offline_section_keep_trying
import indelible.composeapp.generated.resources.offline_unreachable_title
import org.jetbrains.compose.resources.pluralStringResource
import org.jetbrains.compose.resources.stringResource

/** Failed changes as cards, each blocked dependant nested under the create it waits on. */
internal fun LazyListScope.failureRows(
    changes: List<PendingChange>,
    actions: SyncActions,
) {
    val rows = changes.flatMap { change -> listOf(change to false) + change.blocked.map { it to true } }
    itemsIndexed(rows, key = { _, (change, _) -> "change:${change.id}" }) { index, (change, nested) ->
        val edges = CardEdges(first = index == 0, last = index == rows.lastIndex)
        if (nested) ChangeRow(change, nested = true, edges, actions.onRetry) else FailureRow(change, edges, actions)
    }
}

@Composable
private fun FailureRow(
    change: PendingChange,
    edges: CardEdges,
    actions: SyncActions,
) {
    var confirmDiscard by remember { mutableStateOf(false) }
    if (confirmDiscard) {
        DiscardDialog(
            dependants = change.blocked.size,
            onConfirm = {
                confirmDiscard = false
                actions.onDiscard(change.id)
            },
            onDismiss = { confirmDiscard = false },
        )
    }
    val colors = MaterialTheme.colorScheme
    Column(modifier = Modifier.changeCard(edges)) {
        Text(text = failureTitle(change), style = MaterialTheme.typography.titleSmall, color = colors.onSurface)
        Text(
            text = changeMeta(change),
            style = storageMono(TRACK_META, FontWeight.Normal),
            color = IndelibleTheme.colors.textTertiary,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
            modifier = Modifier.padding(top = IndelibleSpacing.step4),
        )
        change.copyText?.let { text ->
            Text(
                text = text,
                style = MaterialTheme.typography.bodyMedium,
                color = colors.onSurface,
                modifier = Modifier.padding(top = IndelibleSpacing.step8),
            )
        }
        failureLine(change)?.let { line ->
            Text(
                text = line,
                style = MaterialTheme.typography.bodySmall,
                color = colors.error,
                modifier = Modifier.padding(top = IndelibleSpacing.step8),
            )
        }
        FailureActions(change, actions, onDiscard = { confirmDiscard = true })
    }
}

@Composable
private fun FailureActions(
    change: PendingChange,
    actions: SyncActions,
    onDiscard: () -> Unit,
) {
    val conflict = change.failure?.kind == SyncFailure.Kind.NOTE_CONFLICT
    if (change.copyText == null && !conflict && !change.canDiscard) return
    Row(
        horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step8),
        modifier = Modifier.padding(top = IndelibleSpacing.step10),
    ) {
        change.copyText?.let { text ->
            RowAction(stringResource(Res.string.offline_action_copy_text), onClick = { actions.onCopyText(text) })
        }
        if (conflict) {
            val viewBoth = stringResource(Res.string.offline_action_view_both)
            RowAction(viewBoth, onClick = { actions.onViewBoth(change.id) }, accent = true)
        }
        if (change.canRetry) {
            RowAction(stringResource(Res.string.common_retry), onClick = { actions.onRetry(change.id) }, accent = true)
        }
        if (change.canDiscard) RowAction(stringResource(Res.string.offline_action_discard), onClick = onDiscard)
    }
}

@Composable
private fun failureTitle(change: PendingChange): String =
    when (change.failure?.kind) {
        SyncFailure.Kind.NOTE_CONFLICT -> stringResource(Res.string.offline_failure_conflict_title)
        SyncFailure.Kind.DELETED_ELSEWHERE -> stringResource(Res.string.offline_failure_deleted_title)
        SyncFailure.Kind.ANCHOR_LOST -> stringResource(Res.string.offline_failure_anchor_title)
        SyncFailure.Kind.REJECTED -> stringResource(Res.string.offline_failure_rejected_title)
        SyncFailure.Kind.SERVER_TOO_OLD -> stringResource(Res.string.offline_failure_too_old_title)
        SyncFailure.Kind.LOCAL_ERROR, null -> stringResource(Res.string.offline_queue_failed)
    }

/** The server's own words where it gave any; the copied text and the conflict stand on their own. */
@Composable
private fun failureLine(change: PendingChange): String? {
    val failure = change.failure ?: return change.error
    return when (failure.kind) {
        SyncFailure.Kind.SERVER_TOO_OLD ->
            stringResource(Res.string.offline_failure_too_old_line, featureName(failure.feature))
        SyncFailure.Kind.REJECTED, SyncFailure.Kind.LOCAL_ERROR -> failure.serverMessage ?: change.error
        SyncFailure.Kind.NOTE_CONFLICT, SyncFailure.Kind.DELETED_ELSEWHERE, SyncFailure.Kind.ANCHOR_LOST -> null
    }
}

@Composable
private fun featureName(feature: SyncFailure.Feature?): String =
    when (feature) {
        SyncFailure.Feature.READING_EVENTS -> stringResource(Res.string.offline_feature_reading_events)
        SyncFailure.Feature.HIGHLIGHTS -> stringResource(Res.string.offline_feature_highlights)
        SyncFailure.Feature.NOTES, null -> stringResource(Res.string.offline_feature_notes)
    }

@Composable
private fun DiscardDialog(
    dependants: Int,
    onConfirm: () -> Unit,
    onDismiss: () -> Unit,
) {
    AlertDialog(
        onDismissRequest = onDismiss,
        title = { Text(stringResource(Res.string.offline_discard_title)) },
        text = {
            Text(
                if (dependants == 0) {
                    stringResource(Res.string.offline_discard_body_alone)
                } else {
                    pluralStringResource(Res.plurals.offline_discard_body_dependants, dependants, dependants)
                },
            )
        },
        confirmButton = {
            TextButton(onClick = onConfirm) {
                Text(
                    text = stringResource(Res.string.offline_action_delete_permanently),
                    color = MaterialTheme.colorScheme.error,
                )
            }
        },
        dismissButton = {
            TextButton(onClick = onDismiss) { Text(stringResource(Res.string.offline_action_keep_it)) }
        },
    )
}

/** The phone is online and the server gave no answer: the queue waits, and one tap asks again. */
@Composable
internal fun UnreachableRow(onRetryNow: () -> Unit) {
    val colors = MaterialTheme.colorScheme
    Column(modifier = Modifier.changeCard(CardEdges(first = true, last = true))) {
        Text(
            text = stringResource(Res.string.offline_unreachable_title),
            style = MaterialTheme.typography.titleSmall,
            color = colors.error,
        )
        Text(
            text = stringResource(Res.string.offline_section_keep_trying),
            style = storageMono(TRACK_META, FontWeight.Normal),
            color = IndelibleTheme.colors.textTertiary,
            modifier = Modifier.padding(top = IndelibleSpacing.step4),
        )
        Row(modifier = Modifier.padding(top = IndelibleSpacing.step10)) {
            RowAction(stringResource(Res.string.offline_action_retry_now), onClick = onRetryNow, accent = true)
        }
    }
}

@Composable
internal fun GroupLabel(text: String) {
    Text(
        text = text,
        style = storageMono(TRACK_LABEL),
        color = IndelibleTheme.colors.textTertiary,
        modifier =
            Modifier.padding(
                start = IndelibleSpacing.rowPaddingH,
                end = IndelibleSpacing.rowPaddingH,
                top = IndelibleSpacing.step20,
                bottom = IndelibleSpacing.step8,
            ),
    )
}

internal class CardEdges(
    val first: Boolean,
    val last: Boolean,
)

@Composable
internal fun Modifier.changeCard(
    edges: CardEdges,
    nested: Boolean = false,
): Modifier {
    val colors = MaterialTheme.colorScheme
    return padding(horizontal = IndelibleSpacing.rowPaddingH)
        .fillMaxWidth()
        .cardCell(edges.first, edges.last, colors.surfaceVariant, colors.outlineVariant)
        .padding(
            start = if (nested) IndelibleSpacing.step32 else IndelibleSpacing.step14,
            end = IndelibleSpacing.step14,
            top = IndelibleSpacing.step12,
            bottom = IndelibleSpacing.step12,
        )
}
