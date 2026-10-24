package app.indelible.offline.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.minimumInteractiveComponentSize
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import app.indelible.offline.viewmodel.Availability
import app.indelible.offline.viewmodel.DocumentOfflineStatus
import app.indelible.offline.viewmodel.SyncBadge
import app.indelible.ui.theme.IndelibleShape
import app.indelible.ui.theme.IndelibleSpacing
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.offline_badge_cached
import indelible.composeapp.generated.resources.offline_badge_downloading
import indelible.composeapp.generated.resources.offline_badge_kept
import indelible.composeapp.generated.resources.offline_badge_pending
import indelible.composeapp.generated.resources.offline_pending_changes
import indelible.composeapp.generated.resources.offline_status_failed
import indelible.composeapp.generated.resources.offline_status_retrying
import org.jetbrains.compose.resources.pluralStringResource
import org.jetbrains.compose.resources.stringResource

/**
 * A document's availability and sync badges for a list row. A document with nothing on the
 * device and nothing queued shows none, so a long list only marks what the device holds.
 */
@Composable
fun OfflineBadges(
    status: DocumentOfflineStatus?,
    modifier: Modifier = Modifier,
) {
    val availability = status?.availability?.let { availabilityLabel(it) }
    val sync = status?.sync
    if (availability == null && sync == null) return
    Row(
        modifier = modifier,
        horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step6),
    ) {
        availability?.let { Badge(it, MaterialTheme.colorScheme.onSurfaceVariant) }
        sync?.let { Badge(syncLabel(it), syncColor(it)) }
    }
}

/** A dot telling the reader that changes to the open document are still waiting to sync; [onClick] shows them. */
@Composable
fun PendingSyncDot(
    sync: SyncBadge?,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    if (sync == null) return
    val description =
        when (sync) {
            is SyncBadge.Pending -> pluralStringResource(Res.plurals.offline_pending_changes, sync.count, sync.count)
            else -> syncLabel(sync)
        }
    val color = syncColor(sync)
    Box(
        modifier =
            modifier
                .minimumInteractiveComponentSize()
                .clickable(onClick = onClick)
                .semantics { contentDescription = description },
        contentAlignment = Alignment.Center,
    ) {
        Box(
            modifier =
                Modifier
                    .size(IndelibleSpacing.step8)
                    .clip(CircleShape)
                    .background(color),
        )
    }
}

@Composable
private fun availabilityLabel(availability: Availability): String? =
    when (availability) {
        Availability.KEPT -> stringResource(Res.string.offline_badge_kept)
        Availability.CACHED -> stringResource(Res.string.offline_badge_cached)
        Availability.DOWNLOADING -> stringResource(Res.string.offline_badge_downloading)
        Availability.NOT_ON_DEVICE -> null
    }

@Composable
private fun syncLabel(sync: SyncBadge): String =
    when (sync) {
        is SyncBadge.Pending -> pluralStringResource(Res.plurals.offline_badge_pending, sync.count, sync.count)
        SyncBadge.Retrying -> stringResource(Res.string.offline_status_retrying)
        is SyncBadge.Failed -> stringResource(Res.string.offline_status_failed)
    }

@Composable
private fun syncColor(sync: SyncBadge): Color =
    if (sync is SyncBadge.Failed) MaterialTheme.colorScheme.error else MaterialTheme.colorScheme.primary

@Composable
private fun Badge(
    text: String,
    color: Color,
) {
    Text(
        text = text,
        style = MaterialTheme.typography.labelSmall,
        color = color,
        modifier =
            Modifier
                .border(IndelibleSpacing.hairline, MaterialTheme.colorScheme.outlineVariant, IndelibleShape.xs)
                .padding(horizontal = IndelibleSpacing.step6, vertical = IndelibleSpacing.step2),
    )
}
