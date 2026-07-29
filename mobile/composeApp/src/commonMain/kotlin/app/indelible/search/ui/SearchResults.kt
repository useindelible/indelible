package app.indelible.search.ui

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Search
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.pulltorefresh.PullToRefreshBox
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.snapshotFlow
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import app.indelible.search.ui.components.SearchResultRow
import app.indelible.search.viewmodel.SearchState
import app.indelible.ui.theme.IndelibleSpacing
import kotlinx.coroutines.flow.distinctUntilChanged

private const val PAGINATION_TRIGGER_ROWS = 5

@OptIn(ExperimentalMaterial3Api::class)
@Composable
internal fun ResultsContent(
    state: SearchState,
    onLoadMore: () -> Unit,
    onRefresh: () -> Unit,
    onResultClick: (String) -> Unit,
    modifier: Modifier = Modifier,
) {
    when {
        state.isSearching -> {
            Box(modifier = modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                CircularProgressIndicator()
            }
        }

        state.error != null -> {
            Box(modifier = modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                Text(
                    text = state.error,
                    style = MaterialTheme.typography.bodyLarge,
                    color = MaterialTheme.colorScheme.error,
                )
            }
        }

        state.results.isEmpty() -> {
            Box(
                modifier =
                    modifier
                        .fillMaxSize()
                        .padding(IndelibleSpacing.step40),
                contentAlignment = Alignment.TopCenter,
            ) {
                Column(horizontalAlignment = Alignment.CenterHorizontally) {
                    Icon(
                        imageVector = Icons.Filled.Search,
                        contentDescription = null,
                        modifier = Modifier.size(IndelibleSpacing.step40),
                        tint = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.4f),
                    )
                    Spacer(modifier = Modifier.height(IndelibleSpacing.step16))
                    Text(
                        text = "No results found",
                        style = MaterialTheme.typography.headlineSmall,
                        color = MaterialTheme.colorScheme.onSurface,
                    )
                    Spacer(modifier = Modifier.height(IndelibleSpacing.step8))
                    Text(
                        text = "Try different keywords, or use filters like tag: or type:",
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
            }
        }

        else -> {
            PullToRefreshBox(
                isRefreshing = state.isRefreshing,
                onRefresh = onRefresh,
                modifier = modifier.fillMaxSize(),
            ) {
                ResultsList(
                    state = state,
                    onLoadMore = onLoadMore,
                    onResultClick = onResultClick,
                )
            }
        }
    }
}

@Composable
private fun ResultsList(
    state: SearchState,
    onLoadMore: () -> Unit,
    onResultClick: (String) -> Unit,
    modifier: Modifier = Modifier,
) {
    val listState = rememberLazyListState()

    LaunchedEffect(listState) {
        snapshotFlow {
            val total = listState.layoutInfo.totalItemsCount
            val last =
                listState.layoutInfo.visibleItemsInfo
                    .lastOrNull()
                    ?.index ?: 0
            total to last
        }.distinctUntilChanged().collect { (total, last) ->
            if (total > 0 && last >= total - PAGINATION_TRIGGER_ROWS) {
                onLoadMore()
            }
        }
    }

    LazyColumn(state = listState, modifier = modifier.fillMaxSize()) {
        item {
            Text(
                text = "${state.results.size}${if (state.hasMore) "+" else ""} results",
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                modifier =
                    Modifier.padding(
                        horizontal = IndelibleSpacing.rowPaddingH,
                        vertical = IndelibleSpacing.step12,
                    ),
            )
        }
        items(items = state.results, key = { it.documentId ?: it.deliveryId ?: it.title }) { result ->
            SearchResultRow(
                result = result,
                onClick = { result.documentId?.let(onResultClick) },
                showDivider = true,
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
                    CircularProgressIndicator(modifier = Modifier.size(IndelibleSpacing.step24))
                }
            }
        }
    }
}
