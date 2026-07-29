package app.indelible.feed.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.BookmarkAdd
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Surface
import androidx.compose.material3.SwipeToDismissBox
import androidx.compose.material3.SwipeToDismissBoxValue
import androidx.compose.material3.Text
import androidx.compose.material3.pulltorefresh.PullToRefreshBox
import androidx.compose.material3.rememberSwipeToDismissBoxState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.runtime.snapshotFlow
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.tooling.preview.Preview
import app.indelible.feed.model.FeedItemWithState
import app.indelible.feed.ui.components.FeedItemRow
import app.indelible.feed.ui.components.FeedScopePopover
import app.indelible.feed.ui.components.FeedTopBar
import app.indelible.feed.viewmodel.FeedEffect
import app.indelible.feed.viewmodel.FeedFilter
import app.indelible.feed.viewmodel.FeedUiState
import app.indelible.feed.viewmodel.FeedViewModel
import app.indelible.ui.theme.AppTheme
import app.indelible.ui.theme.IndelibleSpacing
import app.indelible.ui.theme.IndelibleTheme
import kotlinx.coroutines.flow.distinctUntilChanged

private const val PAGINATION_TRIGGER_ROWS = 5

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun FeedScreen(
    viewModel: FeedViewModel,
    onNavigateToAddFeed: () -> Unit,
    onMenuClick: () -> Unit,
    onProfileClick: () -> Unit,
    onManageSources: () -> Unit,
    onNavigateToReader: (String) -> Unit,
    modifier: Modifier = Modifier,
    userDisplayName: String? = null,
    avatarUrl: String? = null,
    avatarBytes: ByteArray? = null,
) {
    val uiState by viewModel.uiState.collectAsState()
    val feedFilter by viewModel.feedFilter.collectAsState()
    val snackbarHostState = remember { SnackbarHostState() }
    var popoverOpen by remember { mutableStateOf(false) }

    LaunchedEffect(viewModel) {
        viewModel.effects.collect { effect ->
            when (effect) {
                is FeedEffect.ShowSnackbar -> snackbarHostState.showSnackbar(effect.message)
                is FeedEffect.NavigateToAddFeed -> onNavigateToAddFeed()
                is FeedEffect.OpenReader -> onNavigateToReader(effect.documentId)
            }
        }
    }

    val successState = uiState as? FeedUiState.Success

    Scaffold(
        modifier = modifier,
        contentWindowInsets = WindowInsets(0, 0, 0, 0),
        snackbarHost = { SnackbarHost(snackbarHostState) },
    ) { paddingValues ->
        Box(
            modifier =
                Modifier
                    .fillMaxSize()
                    .padding(paddingValues),
        ) {
            Column(modifier = Modifier.fillMaxSize()) {
                FeedTopBar(
                    scopeTitle = feedScopeTitle(feedFilter),
                    sourceCount = successState?.sourceCount,
                    sourceCountExact = successState?.sourceCountExact ?: true,
                    popoverOpen = popoverOpen,
                    userDisplayName = userDisplayName,
                    avatarUrl = avatarUrl,
                    avatarBytes = avatarBytes,
                    onMenuClick = onMenuClick,
                    onScopeClick = { popoverOpen = !popoverOpen },
                    onManageSources = onManageSources,
                    onProfileClick = onProfileClick,
                )

                when (val state = uiState) {
                    is FeedUiState.Loading ->
                        Box(
                            modifier = Modifier.fillMaxSize(),
                            contentAlignment = Alignment.Center,
                        ) {
                            CircularProgressIndicator()
                        }

                    is FeedUiState.Error ->
                        Box(
                            modifier = Modifier.fillMaxSize(),
                            contentAlignment = Alignment.Center,
                        ) {
                            Text(
                                text = state.message,
                                style = MaterialTheme.typography.bodyLarge,
                                color = MaterialTheme.colorScheme.error,
                            )
                        }

                    is FeedUiState.Success ->
                        PullToRefreshBox(
                            isRefreshing = state.isRefreshing,
                            onRefresh = { viewModel.refresh() },
                            modifier = Modifier.fillMaxSize(),
                        ) {
                            if (state.items.isEmpty()) {
                                FeedEmptyState(
                                    filter = feedFilter,
                                    hasSubscriptions = state.hasSubscriptions,
                                    onAddFeed = onNavigateToAddFeed,
                                )
                            } else {
                                FeedItemList(
                                    state = state,
                                    filter = feedFilter,
                                    onSave = { viewModel.saveToLibrary(it) },
                                    onSwipeSave = { item ->
                                        viewModel.saveToLibrary(item)
                                        viewModel.markSeen(item)
                                    },
                                    onOpen = { viewModel.openDelivery(it.id) },
                                    onMarkAllSeen = { viewModel.markAllSeen() },
                                    onLoadMore = { viewModel.loadNextPage() },
                                    modifier = Modifier.fillMaxSize(),
                                )
                            }
                        }
                }
            }

            FeedScopePopover(
                visible = popoverOpen,
                currentFilter = feedFilter,
                onSelectFilter = {
                    popoverOpen = false
                    viewModel.setFeedFilter(it)
                },
                onDismiss = { popoverOpen = false },
            )
        }
    }
}

private fun feedScopeTitle(filter: FeedFilter): String =
    when (filter) {
        FeedFilter.UNSEEN -> "Unseen"
        FeedFilter.SEEN -> "Seen"
    }

@Composable
private fun FeedEmptyState(
    filter: FeedFilter,
    hasSubscriptions: Boolean,
    onAddFeed: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Box(
        modifier = modifier.fillMaxSize(),
        contentAlignment = Alignment.Center,
    ) {
        Column(
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.spacedBy(IndelibleSpacing.step16),
            modifier = Modifier.padding(horizontal = IndelibleSpacing.screenPaddingH),
        ) {
            val message =
                if (!hasSubscriptions) {
                    "Subscribe to sources to see feed updates here."
                } else if (filter == FeedFilter.UNSEEN) {
                    "You're all caught up."
                } else {
                    "No seen items yet."
                }
            Text(
                text = message,
                style = MaterialTheme.typography.bodyLarge,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
            if (!hasSubscriptions) {
                OutlinedButton(onClick = onAddFeed) {
                    Text(
                        text = "Add Feed",
                        style = MaterialTheme.typography.titleSmall,
                    )
                }
            }
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun FeedItemList(
    state: FeedUiState.Success,
    filter: FeedFilter,
    onSave: (FeedItemWithState) -> Unit,
    onSwipeSave: (FeedItemWithState) -> Unit,
    onOpen: (FeedItemWithState) -> Unit,
    onMarkAllSeen: () -> Unit,
    onLoadMore: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val listState = rememberLazyListState()

    LaunchedEffect(listState) {
        snapshotFlow {
            val totalItems = listState.layoutInfo.totalItemsCount
            val lastVisible =
                listState.layoutInfo.visibleItemsInfo
                    .lastOrNull()
                    ?.index ?: 0
            totalItems to lastVisible
        }.distinctUntilChanged().collect { (total, last) ->
            if (total > 0 && last >= total - PAGINATION_TRIGGER_ROWS) {
                onLoadMore()
            }
        }
    }

    LazyColumn(
        state = listState,
        modifier = modifier,
    ) {
        items(
            items = state.items,
            key = { it.id },
        ) { item ->
            FeedItemSwipeableRow(
                item = item,
                saved = item.savedItemId != null || state.savedItemIds.contains(item.id),
                swipeEnabled = filter == FeedFilter.UNSEEN,
                onSave = { onSave(item) },
                onSwipeSave = { onSwipeSave(item) },
                onOpen = { onOpen(item) },
            )
        }
        if (filter == FeedFilter.UNSEEN && state.items.isNotEmpty()) {
            item(key = "mark-all-seen-btn") {
                Box(
                    modifier =
                        Modifier
                            .fillMaxWidth()
                            .padding(vertical = IndelibleSpacing.step16),
                    contentAlignment = Alignment.Center,
                ) {
                    OutlinedButton(onClick = onMarkAllSeen) {
                        Text(
                            text = "Mark all as seen",
                            style = MaterialTheme.typography.titleSmall,
                        )
                    }
                }
            }
        }
        if (state.isLoadingMore) {
            item {
                Box(
                    modifier = Modifier.fillMaxWidth(),
                    contentAlignment = Alignment.Center,
                ) {
                    CircularProgressIndicator(modifier = Modifier.size(IndelibleSpacing.step24))
                }
            }
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun FeedItemSwipeableRow(
    item: FeedItemWithState,
    saved: Boolean,
    swipeEnabled: Boolean,
    onSave: () -> Unit,
    onSwipeSave: () -> Unit,
    onOpen: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val dismissState = rememberSwipeToDismissBoxState()

    LaunchedEffect(dismissState.currentValue) {
        if (dismissState.currentValue == SwipeToDismissBoxValue.StartToEnd) {
            onSwipeSave()
        }
    }

    SwipeToDismissBox(
        state = dismissState,
        modifier = modifier,
        enableDismissFromStartToEnd = swipeEnabled,
        enableDismissFromEndToStart = false,
        backgroundContent = {
            if (dismissState.dismissDirection == SwipeToDismissBoxValue.StartToEnd) {
                Box(
                    modifier =
                        Modifier
                            .fillMaxSize()
                            .background(IndelibleTheme.colors.success),
                    contentAlignment = Alignment.CenterStart,
                ) {
                    Column(
                        horizontalAlignment = Alignment.CenterHorizontally,
                        modifier = Modifier.padding(start = IndelibleSpacing.step24),
                    ) {
                        Icon(
                            imageVector = Icons.Filled.BookmarkAdd,
                            contentDescription = "Save to Library",
                            tint = IndelibleTheme.colors.onSuccess,
                        )
                        Text(
                            text = "Save",
                            style = MaterialTheme.typography.bodySmall,
                            color = IndelibleTheme.colors.onSuccess,
                        )
                    }
                }
            }
        },
    ) {
        FeedItemRow(
            item = item,
            saved = saved,
            onSave = onSave,
            onOpen = onOpen,
        )
    }
}

@Preview(showBackground = true)
@Composable
private fun FeedScreenLoadingPreviewLight() {
    AppTheme(darkTheme = false) {
        Surface(modifier = Modifier.fillMaxSize()) {
            Box(contentAlignment = Alignment.Center, modifier = Modifier.fillMaxSize()) {
                CircularProgressIndicator()
            }
        }
    }
}

@Preview(showBackground = true, uiMode = 0x20)
@Composable
private fun FeedScreenLoadingPreviewDark() {
    AppTheme(darkTheme = true) {
        Surface(modifier = Modifier.fillMaxSize()) {
            Box(contentAlignment = Alignment.Center, modifier = Modifier.fillMaxSize()) {
                CircularProgressIndicator()
            }
        }
    }
}
