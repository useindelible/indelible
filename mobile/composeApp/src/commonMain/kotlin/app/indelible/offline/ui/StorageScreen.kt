package app.indelible.offline.ui

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
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalClipboardManager
import androidx.compose.ui.text.AnnotatedString
import app.indelible.offline.viewmodel.PendingChange
import app.indelible.offline.viewmodel.StorageNotice
import app.indelible.offline.viewmodel.StorageUiState
import app.indelible.offline.viewmodel.StorageViewModel
import app.indelible.ui.components.IndelibleButton
import app.indelible.ui.components.IndelibleButtonStyle
import app.indelible.ui.theme.IndelibleSpacing
import app.indelible.ui.theme.IndelibleTheme
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.common_back
import indelible.composeapp.generated.resources.common_cancel
import indelible.composeapp.generated.resources.offline_notice_changed
import indelible.composeapp.generated.resources.offline_notice_kept_mine
import indelible.composeapp.generated.resources.offline_notice_kept_other
import indelible.composeapp.generated.resources.offline_remove
import indelible.composeapp.generated.resources.offline_remove_all
import indelible.composeapp.generated.resources.offline_remove_downloads_body
import indelible.composeapp.generated.resources.offline_remove_downloads_none_queued
import indelible.composeapp.generated.resources.offline_remove_downloads_title
import indelible.composeapp.generated.resources.offline_title
import kotlinx.datetime.Instant
import org.jetbrains.compose.resources.StringResource
import org.jetbrains.compose.resources.pluralStringResource
import org.jetbrains.compose.resources.stringResource

class StorageActions(
    val onOpenReader: (String) -> Unit = {},
    val onUnpin: (String) -> Unit = {},
    val onRemoveCopy: (String) -> Unit = {},
    val onCapChange: (Long) -> Unit = {},
    val onRemoveDownloads: () -> Unit = {},
    val sync: SyncActions = SyncActions(),
)

/** What the user can do to a queued change; a copied text goes to the clipboard, the rest to the queue. */
class SyncActions(
    val onRetry: (String) -> Unit = {},
    val onRetryNow: () -> Unit = {},
    val onDiscard: (String) -> Unit = {},
    val onCopyText: (String) -> Unit = {},
    val onKeepMine: (changeId: String, seenServerUpdatedAt: Long) -> Unit = { _, _ -> },
    val onKeepOther: (changeId: String, seenServerUpdatedAt: Long) -> Unit = { _, _ -> },
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
    val noticeTexts = StorageNotice.entries.associateWith { stringResource(it.text()) }
    val snackbarHostState = remember { SnackbarHostState() }
    LaunchedEffect(viewModel) {
        viewModel.notices.collect { snackbarHostState.showSnackbar(noticeTexts.getValue(it)) }
    }
    val clipboard = LocalClipboardManager.current
    val actions =
        remember(viewModel, onOpenReader, clipboard) {
            StorageActions(
                onOpenReader = onOpenReader,
                onUnpin = viewModel::unpin,
                onRemoveCopy = viewModel::removeCopy,
                onCapChange = viewModel::setCap,
                onRemoveDownloads = viewModel::removeDownloads,
                sync =
                    SyncActions(
                        onRetry = viewModel::retry,
                        onRetryNow = viewModel::retryNow,
                        onDiscard = viewModel::discard,
                        onCopyText = { clipboard.setText(AnnotatedString(it)) },
                        onKeepMine = viewModel::keepMine,
                        onKeepOther = viewModel::keepOther,
                    ),
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
        snackbarHost = { SnackbarHost(snackbarHostState) },
    ) { padding ->
        StorageContent(state, actions, Modifier.padding(top = padding.calculateTopPadding()))
    }
}

private fun StorageNotice.text(): StringResource =
    when (this) {
        StorageNotice.KEPT_MINE -> Res.string.offline_notice_kept_mine
        StorageNotice.KEPT_OTHER -> Res.string.offline_notice_kept_other
        StorageNotice.CHANGED_REFRESHED -> Res.string.offline_notice_changed
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
            usedBytes = state.usedBytes,
            queuedChanges = state.queuedChanges,
            onConfirm = {
                confirmRemoval = false
                actions.onRemoveDownloads()
            },
            onDismiss = { confirmRemoval = false },
        )
    }
    var conflictId by rememberSaveable { mutableStateOf<String?>(null) }
    state.changes.firstOrNull { it.id == conflictId }?.let { change ->
        ConflictSheet(change, actions.sync, onDismiss = { conflictId = null })
    }
    var filter by remember { mutableStateOf(DownloadFilter.ALL) }
    LazyColumn(modifier = modifier.fillMaxSize()) {
        item { UsageSection(state, actions.onCapChange, onManageKept = { filter = DownloadFilter.KEPT }) }
        documentSections(state, filter, onFilterChange = { filter = it }, actions = actions)
        pendingSection(state.changes, state.serverUnreachable, actions.sync, onViewBoth = { conflictId = it.id })
        if (state.kept.isNotEmpty() || state.cached.isNotEmpty()) {
            item {
                IndelibleButton(
                    text = stringResource(Res.string.offline_remove_all),
                    onClick = { confirmRemoval = true },
                    style = IndelibleButtonStyle.OutlinedDestructive,
                    modifier =
                        Modifier
                            .padding(
                                start = IndelibleSpacing.rowPaddingH,
                                end = IndelibleSpacing.rowPaddingH,
                                top = SECTION_SPACING,
                            ).fillMaxWidth(),
                )
            }
        }
        item { Spacer(modifier = Modifier.height(IndelibleSpacing.step32)) }
    }
}

@Composable
private fun ConflictSheet(
    change: PendingChange,
    actions: SyncActions,
    onDismiss: () -> Unit,
) {
    val theirs = change.failure?.serverNoteBody ?: return
    val theirsAt = change.failure.serverNoteUpdatedAtEpochMs ?: return
    NoteConflictSheet(
        mine = change.detail.orEmpty(),
        theirs = theirs,
        theirsAt = Instant.fromEpochMilliseconds(theirsAt),
        onKeepMine = {
            onDismiss()
            actions.onKeepMine(change.id, theirsAt)
        },
        onKeepOther = {
            onDismiss()
            actions.onKeepOther(change.id, theirsAt)
        },
        onDismiss = onDismiss,
    )
}

/** The quiet line under a section: an empty list, or what the section means. */
@Composable
internal fun StorageNote(text: String) {
    Text(
        text = text,
        style = MaterialTheme.typography.bodySmall,
        color = IndelibleTheme.colors.textTertiary,
        modifier =
            Modifier.padding(
                start = IndelibleSpacing.rowPaddingH,
                end = IndelibleSpacing.rowPaddingH,
                top = IndelibleSpacing.step12,
            ),
    )
}

@Composable
private fun RemoveDownloadsDialog(
    usedBytes: Long,
    queuedChanges: Int,
    onConfirm: () -> Unit,
    onDismiss: () -> Unit,
) {
    AlertDialog(
        onDismissRequest = onDismiss,
        title = { Text(stringResource(Res.string.offline_remove_downloads_title, byteSize(usedBytes))) },
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
