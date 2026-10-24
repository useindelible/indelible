package app.indelible.reader.ui.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import app.indelible.core.i18n.relativeTimeText
import app.indelible.core.offline.Acquisition
import app.indelible.core.offline.DownloadFailure
import app.indelible.offline.ui.byteSize
import app.indelible.offline.viewmodel.Availability
import app.indelible.offline.viewmodel.DocumentOfflineStatus
import app.indelible.offline.viewmodel.DocumentOfflineViewModel
import app.indelible.ui.components.IndelibleButton
import app.indelible.ui.components.IndelibleButtonStyle
import app.indelible.ui.theme.IndelibleSpacing
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.offline_badge_downloading
import indelible.composeapp.generated.resources.offline_download_failed
import indelible.composeapp.generated.resources.offline_keep
import indelible.composeapp.generated.resources.offline_manage_storage
import indelible.composeapp.generated.resources.offline_no_space
import indelible.composeapp.generated.resources.offline_not_on_device
import indelible.composeapp.generated.resources.offline_not_synced
import indelible.composeapp.generated.resources.offline_remove_copy
import indelible.composeapp.generated.resources.offline_synced
import indelible.composeapp.generated.resources.offline_waiting
import indelible.composeapp.generated.resources.offline_waiting_connection
import kotlinx.datetime.Clock
import kotlinx.datetime.Instant
import org.jetbrains.compose.resources.stringResource

/** The item record's offline section: its status and what the user can do about the copy. */
class ItemOffline(
    val status: DocumentOfflineStatus,
    val onKeepOffline: (Boolean) -> Unit = {},
    val onRemove: () -> Unit = {},
    val onOpenStorage: () -> Unit = {},
)

/** The item record's offline section for [viewModel]'s document, following its status; none without one. */
@Composable
fun rememberItemOffline(
    viewModel: DocumentOfflineViewModel?,
    onOpenStorage: () -> Unit,
): ItemOffline? {
    if (viewModel == null) return null
    val status by viewModel.status.collectAsState()
    return remember(status, viewModel, onOpenStorage) {
        ItemOffline(status, viewModel::setKeepOffline, viewModel::removeFromDevice, onOpenStorage)
    }
}

@Composable
fun ItemOfflineSection(
    offline: ItemOffline,
    modifier: Modifier = Modifier,
) {
    val status = offline.status
    Column(
        modifier = modifier.fillMaxWidth(),
        verticalArrangement = Arrangement.spacedBy(IndelibleSpacing.step8),
    ) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            Text(
                text = stringResource(Res.string.offline_keep),
                style = MaterialTheme.typography.bodyLarge,
                modifier = Modifier.weight(1f),
            )
            // A download under way is already a copy on its way, so the switch shows it as kept.
            Switch(
                checked = status.availability == Availability.KEPT || status.availability == Availability.DOWNLOADING,
                onCheckedChange = offline.onKeepOffline,
            )
        }
        OfflineNote(copySummary(status))
        AcquisitionLine(status.acquisition, offline.onOpenStorage)
        if (status.bytes != null) {
            IndelibleButton(
                text = stringResource(Res.string.offline_remove_copy),
                onClick = offline.onRemove,
                style = IndelibleButtonStyle.Secondary,
            )
        }
    }
}

@Composable
private fun copySummary(status: DocumentOfflineStatus): String {
    val bytes = status.bytes
    val syncedAt = status.lastSyncedAt
    return when {
        bytes == null -> stringResource(Res.string.offline_not_on_device)
        syncedAt == null -> stringResource(Res.string.offline_not_synced, byteSize(bytes))
        else -> {
            val synced = relativeTimeText(Instant.fromEpochMilliseconds(syncedAt), Clock.System.now())
            stringResource(Res.string.offline_synced, byteSize(bytes), synced)
        }
    }
}

@Composable
private fun AcquisitionLine(
    acquisition: Acquisition?,
    onOpenStorage: () -> Unit,
) {
    when (acquisition) {
        null -> Unit
        Acquisition.Waiting -> OfflineNote(stringResource(Res.string.offline_waiting))
        Acquisition.WaitingForConnection -> OfflineNote(stringResource(Res.string.offline_waiting_connection))
        is Acquisition.Downloading -> {
            OfflineNote(stringResource(Res.string.offline_badge_downloading))
            LinearProgressIndicator(progress = { acquisition.fraction }, modifier = Modifier.fillMaxWidth())
        }
        is Acquisition.Failed ->
            if (acquisition.reason == DownloadFailure.NO_SPACE) {
                Row(verticalAlignment = Alignment.CenterVertically) {
                    OfflineNote(stringResource(Res.string.offline_no_space), isError = true)
                    IndelibleButton(
                        text = stringResource(Res.string.offline_manage_storage),
                        onClick = onOpenStorage,
                        style = IndelibleButtonStyle.Text,
                        compact = true,
                    )
                }
            } else {
                OfflineNote(stringResource(Res.string.offline_download_failed), isError = true)
            }
    }
}

@Composable
private fun OfflineNote(
    text: String,
    isError: Boolean = false,
) {
    Text(
        text = text,
        style = MaterialTheme.typography.bodySmall,
        color = if (isError) MaterialTheme.colorScheme.error else MaterialTheme.colorScheme.onSurfaceVariant,
    )
}
