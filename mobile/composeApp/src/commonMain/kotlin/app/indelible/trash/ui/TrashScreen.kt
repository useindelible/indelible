package app.indelible.trash.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.pulltorefresh.PullToRefreshBox
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.snapshotFlow
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import app.indelible.core.model.LibraryItem
import app.indelible.core.model.ThumbnailColor
import app.indelible.trash.viewmodel.TrashState
import app.indelible.trash.viewmodel.TrashViewModel
import app.indelible.ui.theme.IndelibleIcons
import app.indelible.ui.theme.IndelibleSpacing
import app.indelible.ui.theme.IndelibleTheme
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.map
import kotlinx.datetime.Instant
import kotlinx.datetime.TimeZone
import kotlinx.datetime.toLocalDateTime

private const val PAGINATION_TRIGGER_ROWS = 5
private const val THUMBNAIL_OPACITY = 0.5f

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun TrashScreen(
    viewModel: TrashViewModel,
    onNavigateBack: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val state by viewModel.state.collectAsState()
    val listState = rememberLazyListState()

    LaunchedEffect(listState) {
        snapshotFlow { listState.layoutInfo }
            .map { info ->
                val lastVisible = info.visibleItemsInfo.lastOrNull()?.index ?: 0
                val total = info.totalItemsCount
                total > 0 && lastVisible >= total - PAGINATION_TRIGGER_ROWS
            }.distinctUntilChanged()
            .collect { nearEnd ->
                if (nearEnd) viewModel.loadNextPage()
            }
    }

    if (state.showEmptyConfirm) {
        EmptyTrashDialog(
            itemCount = state.items.size,
            onConfirm = { viewModel.emptyTrash() },
            onDismiss = { viewModel.dismissEmptyConfirm() },
        )
    }

    Scaffold(
        modifier = modifier,
        topBar = {
            TrashTopBar(
                itemCount = state.items.size,
                isEmptying = state.isEmptying,
                onNavigateBack = onNavigateBack,
                onRequestEmptyTrash = { viewModel.requestEmptyTrash() },
            )
        },
    ) { paddingValues ->
        PullToRefreshBox(
            isRefreshing = state.isRefreshing,
            onRefresh = { viewModel.refresh() },
            modifier =
                Modifier
                    .fillMaxSize()
                    .padding(paddingValues),
        ) {
            when {
                state.isLoading && state.items.isEmpty() -> {
                    Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                        CircularProgressIndicator()
                    }
                }

                state.error != null && state.items.isEmpty() -> {
                    Column(
                        modifier = Modifier.fillMaxSize(),
                        horizontalAlignment = Alignment.CenterHorizontally,
                        verticalArrangement = Arrangement.Center,
                    ) {
                        Text(
                            text = state.error ?: "Something went wrong",
                            style = MaterialTheme.typography.bodyMedium,
                            color = MaterialTheme.colorScheme.error,
                        )
                        Spacer(modifier = Modifier.height(IndelibleSpacing.step12))
                        TextButton(onClick = { viewModel.load() }) { Text("Retry") }
                    }
                }

                else -> {
                    TrashItemList(
                        state = state,
                        listState = listState,
                        onRestore = { item -> viewModel.restoreItem(item.id) },
                    )
                }
            }
        }
    }
}

@Composable
private fun EmptyTrashDialog(
    itemCount: Int,
    onConfirm: () -> Unit,
    onDismiss: () -> Unit,
) {
    AlertDialog(
        onDismissRequest = onDismiss,
        title = { Text("Empty Trash?") },
        text = {
            Text(
                text = "All $itemCount item${if (itemCount == 1) "" else "s"}" +
                    " will be permanently deleted. This cannot be undone.",
                style = MaterialTheme.typography.bodyMedium,
            )
        },
        confirmButton = {
            TextButton(onClick = onConfirm) {
                Text(
                    text = "Empty Trash",
                    color = MaterialTheme.colorScheme.error,
                )
            }
        },
        dismissButton = {
            TextButton(onClick = onDismiss) {
                Text("Cancel")
            }
        },
    )
}

@Composable
private fun TrashTopBar(
    itemCount: Int,
    isEmptying: Boolean,
    onNavigateBack: () -> Unit,
    onRequestEmptyTrash: () -> Unit,
) {
    Row(
        modifier =
            Modifier
                .fillMaxWidth()
                .statusBarsPadding(),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        IconButton(
            onClick = onNavigateBack,
            modifier = Modifier.size(IndelibleSpacing.step48),
        ) {
            Icon(
                imageVector = Icons.AutoMirrored.Filled.ArrowBack,
                contentDescription = "Back",
                tint = MaterialTheme.colorScheme.primary,
            )
        }
        Text(
            text = "Trash",
            style = MaterialTheme.typography.titleLarge,
            modifier = Modifier.weight(1f),
        )
        if (itemCount > 0) {
            TextButton(
                onClick = onRequestEmptyTrash,
                enabled = !isEmptying,
            ) {
                if (isEmptying) {
                    CircularProgressIndicator(
                        modifier = Modifier.size(IndelibleSpacing.step16),
                        strokeWidth = IndelibleSpacing.step2,
                    )
                } else {
                    Text(
                        text = "Empty Trash",
                        color = MaterialTheme.colorScheme.error,
                        style = MaterialTheme.typography.bodyMedium,
                    )
                }
            }
        }
    }
}

@Composable
private fun TrashItemList(
    state: TrashState,
    listState: LazyListState,
    onRestore: (LibraryItem) -> Unit,
) {
    LazyColumn(
        state = listState,
        modifier = Modifier.fillMaxSize(),
    ) {
        item {
            WarningBanner(
                message = "Items are permanently deleted after 30 days",
                modifier =
                    Modifier.padding(
                        horizontal = IndelibleSpacing.step16,
                        vertical = IndelibleSpacing.step12,
                    ),
            )
        }

        if (state.items.isEmpty()) {
            item {
                Box(
                    modifier =
                        Modifier
                            .fillMaxWidth()
                            .padding(vertical = IndelibleSpacing.step40),
                    contentAlignment = Alignment.Center,
                ) {
                    Text(
                        text = "Trash is empty",
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
            }
        } else {
            items(items = state.items, key = { it.id }) { item ->
                TrashItemRow(
                    item = item,
                    isRestoring = item.id in state.restoringItemIds,
                    onRestore = { onRestore(item) },
                )
            }

            if (state.isLoadingMore) {
                item {
                    Box(
                        modifier =
                            Modifier
                                .fillMaxWidth()
                                .padding(IndelibleSpacing.step16),
                        contentAlignment = Alignment.Center,
                    ) {
                        CircularProgressIndicator(
                            modifier = Modifier.size(IndelibleSpacing.step24),
                        )
                    }
                }
            }
        }
    }
}

@Composable
private fun WarningBanner(
    message: String,
    modifier: Modifier = Modifier,
) {
    Row(
        modifier =
            modifier
                .fillMaxWidth()
                .clip(MaterialTheme.shapes.medium)
                .background(IndelibleTheme.colors.warning.copy(alpha = 0.12f))
                .padding(IndelibleSpacing.step12),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Icon(
            imageVector = IndelibleIcons.WarningTriangle,
            contentDescription = null,
            tint = IndelibleTheme.colors.warning,
            modifier = Modifier.size(IndelibleSpacing.step20),
        )
        Spacer(modifier = Modifier.width(IndelibleSpacing.step8))
        Text(
            text = message,
            style = MaterialTheme.typography.bodySmall,
            color = IndelibleTheme.colors.warning,
        )
    }
}

@Composable
private fun TrashItemRow(
    item: LibraryItem,
    isRestoring: Boolean,
    onRestore: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Column(modifier = modifier.background(MaterialTheme.colorScheme.surface)) {
        Row(
            modifier =
                Modifier
                    .fillMaxWidth()
                    .padding(
                        horizontal = IndelibleSpacing.rowPaddingH,
                        vertical = IndelibleSpacing.rowPaddingV,
                    ),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Box(modifier = Modifier.alpha(THUMBNAIL_OPACITY)) {
                ThumbnailPlaceholder(
                    item = item,
                    modifier = Modifier.size(IndelibleSpacing.step48),
                )
            }
            Spacer(modifier = Modifier.width(IndelibleSpacing.step14))
            Column(modifier = Modifier.weight(1f)) {
                Text(
                    text = item.title,
                    style = MaterialTheme.typography.titleSmall,
                    color = MaterialTheme.colorScheme.onSurface,
                    maxLines = 1,
                )
                Spacer(modifier = Modifier.height(IndelibleSpacing.step2))
                Text(
                    text = formatDeletedDate(item.deletedAt),
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
            Spacer(modifier = Modifier.width(IndelibleSpacing.step8))
            TextButton(
                onClick = onRestore,
                enabled = !isRestoring,
            ) {
                if (isRestoring) {
                    CircularProgressIndicator(
                        modifier = Modifier.size(IndelibleSpacing.step16),
                        strokeWidth = IndelibleSpacing.step2,
                    )
                } else {
                    Text(
                        text = "Restore",
                        style = MaterialTheme.typography.bodySmall.copy(fontWeight = FontWeight.Medium),
                        color = MaterialTheme.colorScheme.primary,
                    )
                }
            }
        }
        HorizontalDivider(
            color = MaterialTheme.colorScheme.outlineVariant,
            modifier = Modifier.padding(start = IndelibleSpacing.rowPaddingH),
        )
    }
}

@Composable
private fun ThumbnailPlaceholder(
    item: LibraryItem,
    modifier: Modifier = Modifier,
) {
    val thumbnailColor = ThumbnailColor.forId(item.id)
    val backgroundColor = thumbnailBackground(thumbnailColor)

    Box(
        modifier =
            modifier
                .clip(MaterialTheme.shapes.medium)
                .background(backgroundColor),
        contentAlignment = Alignment.Center,
    ) {
        Text(
            text =
                item.title
                    .firstOrNull()
                    ?.uppercaseChar()
                    ?.toString() ?: "?",
            style = MaterialTheme.typography.titleLarge,
            color = MaterialTheme.colorScheme.onSurface.copy(alpha = 0.7f),
        )
    }
}

@Composable
private fun thumbnailBackground(thumbnailColor: ThumbnailColor) =
    when (thumbnailColor) {
        ThumbnailColor.BLUE -> MaterialTheme.colorScheme.primary.copy(alpha = 0.18f)
        ThumbnailColor.GREEN -> IndelibleTheme.colors.success.copy(alpha = 0.18f)
        ThumbnailColor.PURPLE -> MaterialTheme.colorScheme.primaryContainer.copy(alpha = 0.48f)
        ThumbnailColor.ORANGE -> IndelibleTheme.colors.warning.copy(alpha = 0.18f)
        ThumbnailColor.RED -> MaterialTheme.colorScheme.error.copy(alpha = 0.18f)
        ThumbnailColor.TEAL -> MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.68f)
        ThumbnailColor.PINK -> MaterialTheme.colorScheme.error.copy(alpha = 0.20f)
    }

private const val MONTH_ABBREVIATION_LENGTH = 3

private fun formatDeletedDate(instant: Instant?): String {
    if (instant == null) return "Deleted"
    return runCatching {
        val dt = instant.toLocalDateTime(TimeZone.currentSystemDefault())
        val month =
            dt.month.name
                .lowercase()
                .replaceFirstChar { it.uppercase() }
                .take(MONTH_ABBREVIATION_LENGTH)
        "Deleted $month ${dt.dayOfMonth}, ${dt.year}"
    }.getOrElse { "Deleted" }
}
