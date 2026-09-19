package app.indelible.offline.ui

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Slider
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import app.indelible.offline.viewmodel.StorageUiState
import app.indelible.offline.viewmodel.StorageViewModel
import app.indelible.profile.ui.components.SettingsSection
import app.indelible.ui.components.IndelibleButton
import app.indelible.ui.components.IndelibleButtonStyle
import app.indelible.ui.theme.IndelibleSpacing
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.common_back
import indelible.composeapp.generated.resources.common_cancel
import indelible.composeapp.generated.resources.offline_limit
import indelible.composeapp.generated.resources.offline_limit_hint
import indelible.composeapp.generated.resources.offline_remove
import indelible.composeapp.generated.resources.offline_remove_downloads
import indelible.composeapp.generated.resources.offline_remove_downloads_body
import indelible.composeapp.generated.resources.offline_remove_downloads_none_queued
import indelible.composeapp.generated.resources.offline_remove_downloads_title
import indelible.composeapp.generated.resources.offline_title
import indelible.composeapp.generated.resources.offline_used
import org.jetbrains.compose.resources.pluralStringResource
import org.jetbrains.compose.resources.stringResource
import kotlin.math.abs
import kotlin.math.roundToInt

private const val SMALLEST_CAP_BYTES = 256L * 1024 * 1024
private const val CAP_CHOICES = 5

// 256 MB doubling to 4 GB; the default 1 GB is one of them.
private val capChoices = generateSequence(SMALLEST_CAP_BYTES) { it * 2 }.take(CAP_CHOICES).toList()

class StorageActions(
    val onOpenReader: (String) -> Unit = {},
    val onRemoveCopy: (String) -> Unit = {},
    val onCapChange: (Long) -> Unit = {},
    val onRetry: (String) -> Unit = {},
    val onRemoveDownloads: () -> Unit = {},
)

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun StorageScreen(
    viewModel: StorageViewModel,
    onNavigateBack: () -> Unit,
    onOpenReader: (String) -> Unit,
    modifier: Modifier = Modifier,
) {
    val state by viewModel.state.collectAsState()
    val actions =
        remember(viewModel, onOpenReader) {
            StorageActions(
                onOpenReader = onOpenReader,
                onRemoveCopy = viewModel::removeCopy,
                onCapChange = viewModel::setCap,
                onRetry = viewModel::retry,
                onRemoveDownloads = viewModel::removeDownloads,
            )
        }
    Scaffold(
        modifier = modifier,
        topBar = {
            TopAppBar(
                title = {
                    Text(
                        text = stringResource(Res.string.offline_title),
                        style = MaterialTheme.typography.headlineSmall,
                    )
                },
                navigationIcon = {
                    IconButton(onClick = onNavigateBack) {
                        Icon(
                            imageVector = Icons.AutoMirrored.Filled.ArrowBack,
                            contentDescription = stringResource(Res.string.common_back),
                        )
                    }
                },
            )
        },
    ) { padding ->
        StorageContent(state, actions, Modifier.padding(top = padding.calculateTopPadding()))
    }
}

@Composable
fun StorageContent(
    state: StorageUiState,
    actions: StorageActions,
    modifier: Modifier = Modifier,
) {
    var confirmRemoval by remember { mutableStateOf(false) }
    if (confirmRemoval) {
        RemoveDownloadsDialog(
            queuedChanges = state.queuedChanges,
            onConfirm = {
                confirmRemoval = false
                actions.onRemoveDownloads()
            },
            onDismiss = { confirmRemoval = false },
        )
    }
    LazyColumn(modifier = modifier.fillMaxSize()) {
        item { UsageSection(state.usedBytes, state.capBytes, actions.onCapChange) }
        documentSections(state, actions)
        pendingSection(state.changes, actions.onRetry)
        if (state.kept.isNotEmpty() || state.cached.isNotEmpty()) {
            item {
                IndelibleButton(
                    text = stringResource(Res.string.offline_remove_downloads),
                    onClick = { confirmRemoval = true },
                    style = IndelibleButtonStyle.OutlinedDestructive,
                    modifier = Modifier.padding(IndelibleSpacing.rowPaddingH),
                )
            }
        }
        item { Spacer(modifier = Modifier.height(IndelibleSpacing.step32)) }
    }
}

/** A one-line explanation standing in for an empty list. */
@Composable
internal fun StorageNote(text: String) {
    Text(
        text = text,
        style = MaterialTheme.typography.bodyMedium,
        color = MaterialTheme.colorScheme.onSurfaceVariant,
        modifier = Modifier.padding(horizontal = IndelibleSpacing.rowPaddingH, vertical = IndelibleSpacing.step8),
    )
}

@Composable
private fun UsageSection(
    usedBytes: Long,
    capBytes: Long,
    onCapChange: (Long) -> Unit,
) {
    SettingsSection(title = stringResource(Res.string.offline_limit)) {
        Column(
            modifier = Modifier.padding(horizontal = IndelibleSpacing.rowPaddingH),
            verticalArrangement = Arrangement.spacedBy(IndelibleSpacing.step8),
        ) {
            Text(
                text = stringResource(Res.string.offline_used, byteSize(usedBytes), byteSize(capBytes)),
                style = MaterialTheme.typography.bodyMedium,
            )
            LinearProgressIndicator(
                progress = { (usedBytes.toFloat() / capBytes).coerceIn(0f, 1f) },
                modifier = Modifier.fillMaxWidth(),
            )
            CapSlider(capBytes, onCapChange)
            Text(
                text = stringResource(Res.string.offline_limit_hint),
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
    }
}

@Composable
private fun CapSlider(
    capBytes: Long,
    onCapChange: (Long) -> Unit,
) {
    val nearest = capChoices.indices.minBy { abs(capChoices[it] - capBytes) }
    var position by remember(capBytes) { mutableFloatStateOf(nearest.toFloat()) }
    val chosen = capChoices[position.roundToInt()]
    Row(verticalAlignment = Alignment.CenterVertically) {
        Slider(
            value = position,
            onValueChange = { position = it },
            onValueChangeFinished = { onCapChange(chosen) },
            valueRange = 0f..capChoices.lastIndex.toFloat(),
            steps = capChoices.size - 2,
            modifier = Modifier.weight(1f),
        )
        Text(
            text = byteSize(chosen),
            style = MaterialTheme.typography.bodyMedium,
            modifier = Modifier.padding(start = IndelibleSpacing.step8),
        )
    }
}

@Composable
private fun RemoveDownloadsDialog(
    queuedChanges: Int,
    onConfirm: () -> Unit,
    onDismiss: () -> Unit,
) {
    AlertDialog(
        onDismissRequest = onDismiss,
        title = { Text(stringResource(Res.string.offline_remove_downloads_title)) },
        text = {
            Text(
                if (queuedChanges == 0) {
                    stringResource(Res.string.offline_remove_downloads_none_queued)
                } else {
                    pluralStringResource(Res.plurals.offline_remove_downloads_body, queuedChanges, queuedChanges)
                },
            )
        },
        confirmButton = {
            TextButton(onClick = onConfirm) {
                Text(
                    text = stringResource(Res.string.offline_remove),
                    color = MaterialTheme.colorScheme.error,
                )
            }
        },
        dismissButton = {
            TextButton(onClick = onDismiss) { Text(stringResource(Res.string.common_cancel)) }
        },
    )
}
