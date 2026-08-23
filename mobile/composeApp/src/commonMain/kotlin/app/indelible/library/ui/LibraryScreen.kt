package app.indelible.library.ui

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
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.pulltorefresh.PullToRefreshBox
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.runtime.snapshotFlow
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.tooling.preview.Preview
import app.indelible.core.model.LibraryCounts
import app.indelible.library.ui.components.AddUrlBottomSheet
import app.indelible.library.ui.components.ContentTypeFilterRow
import app.indelible.library.ui.components.ItemSwipeableRow
import app.indelible.library.ui.components.LibraryEmptyState
import app.indelible.library.ui.components.LibraryFab
import app.indelible.library.ui.components.LibraryTopBar
import app.indelible.library.ui.components.ScopeSwitcherPopover
import app.indelible.library.viewmodel.LibraryEffect
import app.indelible.library.viewmodel.LibraryScope
import app.indelible.library.viewmodel.LibraryUiState
import app.indelible.library.viewmodel.LibraryViewModel
import app.indelible.profile.viewmodel.AddLibraryEffect
import app.indelible.profile.viewmodel.AddLibraryViewModel
import app.indelible.sidebar.model.Collection
import app.indelible.sidebar.model.SmartList
import app.indelible.ui.theme.AppTheme
import app.indelible.ui.theme.IndelibleSpacing
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.common_coming_soon
import indelible.composeapp.generated.resources.library_scope_archive
import indelible.composeapp.generated.resources.library_scope_inbox
import indelible.composeapp.generated.resources.library_scope_later
import indelible.composeapp.generated.resources.library_url_queued
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.launch
import org.jetbrains.compose.resources.stringResource

private const val PAGINATION_TRIGGER_ROWS = 5

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun LibraryScreen(
    viewModel: LibraryViewModel,
    addLibraryViewModel: AddLibraryViewModel,
    onNavigateToItem: (String) -> Unit,
    onMenuClick: () -> Unit,
    onProfileClick: () -> Unit,
    collections: List<Collection>,
    smartLists: List<SmartList>,
    modifier: Modifier = Modifier,
    userDisplayName: String? = null,
    avatarUrl: String? = null,
    avatarBytes: ByteArray? = null,
) {
    val uiState by viewModel.uiState.collectAsState()
    val addLibraryUiState by addLibraryViewModel.uiState.collectAsState()
    val triageFilter by viewModel.triageFilter.collectAsState()
    val contentTypeFilter by viewModel.contentTypeFilter.collectAsState()
    val counts by viewModel.counts.collectAsState()
    val scope by viewModel.scope.collectAsState()
    val snackbarHostState = remember { SnackbarHostState() }
    val coroutineScope = rememberCoroutineScope()
    var popoverOpen by remember { mutableStateOf(false) }
    var addUrlSheetOpen by remember { mutableStateOf(false) }
    val comingSoonMessage = stringResource(Res.string.common_coming_soon)
    val urlQueuedMessage = stringResource(Res.string.library_url_queued)

    LaunchedEffect(viewModel) {
        viewModel.effects.collect { effect ->
            when (effect) {
                is LibraryEffect.ShowSnackbar -> snackbarHostState.showSnackbar(effect.message)
            }
        }
    }

    LaunchedEffect(addLibraryViewModel) {
        addLibraryViewModel.effects.collect { effect ->
            when (effect) {
                AddLibraryEffect.Saved -> {
                    addUrlSheetOpen = false
                    viewModel.refresh()
                    snackbarHostState.showSnackbar(urlQueuedMessage)
                }
            }
        }
    }

    val scopeTitle = scopeTitle(scope, triageFilter)
    val scopeCount = scopeCount(scope, collections, counts)

    Scaffold(
        modifier = modifier,
        contentWindowInsets = WindowInsets(0, 0, 0, 0),
        floatingActionButton = {
            LibraryFab(
                onClick = {
                    addLibraryViewModel.reset()
                    addUrlSheetOpen = true
                },
            )
        },
        snackbarHost = { SnackbarHost(snackbarHostState) },
    ) { paddingValues ->
        Box(
            modifier =
                Modifier
                    .fillMaxSize()
                    .padding(paddingValues),
        ) {
            Column(modifier = Modifier.fillMaxSize()) {
                LibraryTopBar(
                    scopeTitle = scopeTitle,
                    scopeCount = scopeCount,
                    counts = counts,
                    popoverOpen = popoverOpen,
                    userDisplayName = userDisplayName,
                    avatarUrl = avatarUrl,
                    avatarBytes = avatarBytes,
                    onMenuClick = onMenuClick,
                    onScopeClick = { popoverOpen = !popoverOpen },
                    onSortClick = { coroutineScope.launch { snackbarHostState.showSnackbar(comingSoonMessage) } },
                    onProfileClick = onProfileClick,
                )

                if (scope is LibraryScope.Triage) {
                    ContentTypeFilterRow(
                        selected = contentTypeFilter,
                        counts = counts,
                        onSelect = { viewModel.setContentTypeFilter(it) },
                        modifier = Modifier.padding(bottom = IndelibleSpacing.step16),
                    )
                }

                when (val state = uiState) {
                    is LibraryUiState.Loading ->
                        Box(
                            modifier = Modifier.fillMaxSize(),
                            contentAlignment = Alignment.Center,
                        ) {
                            CircularProgressIndicator()
                        }

                    is LibraryUiState.Error ->
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

                    is LibraryUiState.Success ->
                        PullToRefreshBox(
                            isRefreshing = state.isRefreshing,
                            onRefresh = { viewModel.refresh() },
                            modifier = Modifier.fillMaxSize(),
                        ) {
                            if (state.items.isEmpty()) {
                                LibraryEmptyState(triageFilter = triageFilter)
                            } else {
                                ItemList(
                                    state = state,
                                    onNavigateToItem = onNavigateToItem,
                                    onDelete = { viewModel.deleteItem(it) },
                                    onTriage = { item, triageState -> viewModel.triageItem(item, triageState) },
                                    onLoadMore = { viewModel.loadNextPage() },
                                    modifier = Modifier.fillMaxSize(),
                                )
                            }
                        }
                }
            }

            ScopeSwitcherPopover(
                visible = popoverOpen,
                currentScope = scope,
                currentTriage = triageFilter,
                collections = collections,
                smartLists = smartLists,
                onSelectTriage = {
                    popoverOpen = false
                    viewModel.setTriageFilter(it)
                },
                onSelectCollection = {
                    popoverOpen = false
                    viewModel.setScope(LibraryScope.Collection(it.id, it.name))
                },
                onSelectSmartList = {
                    popoverOpen = false
                    viewModel.setScope(LibraryScope.SmartList(it.id, it.name))
                },
                onDismiss = { popoverOpen = false },
            )
        }
    }

    if (addUrlSheetOpen) {
        AddUrlBottomSheet(
            uiState = addLibraryUiState,
            onSubmit = addLibraryViewModel::save,
            onInputChanged = addLibraryViewModel::clearError,
            onDismiss = {
                if (!addLibraryUiState.isSubmitting) {
                    addUrlSheetOpen = false
                    addLibraryViewModel.reset()
                }
            },
        )
    }
}

@Composable
private fun scopeTitle(
    scope: LibraryScope,
    triageFilter: app.indelible.library.viewmodel.TriageFilter,
): String =
    when (scope) {
        is LibraryScope.Triage ->
            stringResource(
                when (triageFilter) {
                    app.indelible.library.viewmodel.TriageFilter.INBOX -> Res.string.library_scope_inbox
                    app.indelible.library.viewmodel.TriageFilter.LATER -> Res.string.library_scope_later
                    app.indelible.library.viewmodel.TriageFilter.ARCHIVE -> Res.string.library_scope_archive
                },
            )
        is LibraryScope.Collection -> scope.name
        is LibraryScope.SmartList -> scope.name
    }

internal fun scopeCount(
    scope: LibraryScope,
    collections: List<Collection>,
    counts: LibraryCounts?,
): Int? =
    when (scope) {
        is LibraryScope.Triage -> counts?.total
        is LibraryScope.Collection -> collections.find { it.id == scope.id }?.itemCount?.toInt()
        is LibraryScope.SmartList -> null
    }

@Composable
private fun ItemList(
    state: LibraryUiState.Success,
    onNavigateToItem: (String) -> Unit,
    onDelete: (app.indelible.core.model.LibraryItem) -> Unit,
    onTriage: (app.indelible.core.model.LibraryItem, String) -> Unit,
    onLoadMore: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val listState = rememberLazyListState()

    LaunchedEffect(state.items.firstOrNull()?.id) {
        if (state.items.isNotEmpty()) listState.scrollToItem(0)
    }

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
            ItemSwipeableRow(
                item = item,
                onTap = { onNavigateToItem(item.documentId) },
                onDelete = { onDelete(item) },
                onTriage = { triageState -> onTriage(item, triageState) },
                showDivider = true,
                modifier = Modifier.fillMaxWidth(),
            )
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

@Preview(showBackground = true)
@Composable
private fun LibraryScreenLoadingPreviewLight() {
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
private fun LibraryScreenLoadingPreviewDark() {
    AppTheme(darkTheme = true) {
        Surface(modifier = Modifier.fillMaxSize()) {
            Box(contentAlignment = Alignment.Center, modifier = Modifier.fillMaxSize()) {
                CircularProgressIndicator()
            }
        }
    }
}
