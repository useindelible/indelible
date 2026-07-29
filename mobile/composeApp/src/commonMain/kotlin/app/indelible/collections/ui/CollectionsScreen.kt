package app.indelible.collections.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.GridItemSpan
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.foundation.lazy.grid.LazyGridState
import androidx.compose.foundation.lazy.grid.rememberLazyGridState
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.pulltorefresh.PullToRefreshBox
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextOverflow
import app.indelible.api.generated.models.CollectionResponse
import app.indelible.collections.ui.components.CollectionScreenTopBar
import app.indelible.collections.ui.components.GridPaginationEffect
import app.indelible.collections.viewmodel.CollectionsState
import app.indelible.collections.viewmodel.CollectionsViewModel
import app.indelible.ui.theme.IndelibleIcons
import app.indelible.ui.theme.IndelibleSpacing
import app.indelible.ui.theme.IndelibleTheme

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun CollectionsScreen(
    viewModel: CollectionsViewModel,
    onNavigateBack: () -> Unit,
    onNavigateToCollection: (String) -> Unit,
    modifier: Modifier = Modifier,
) {
    val state by viewModel.state.collectAsState()
    val gridState = rememberLazyGridState()

    // Only show top-level collections in the overview — sub-collections belong on detail screens.
    // Sort alphabetically by name, matching the web default sort order.
    val rootCollections = state.collections.filter { it.parentId == null }.sortedBy { it.name.lowercase() }

    GridPaginationEffect(
        gridState = gridState,
        hasMore = state.hasMore,
        onLoadMore = { viewModel.loadNextPage() },
    )

    Scaffold(
        modifier = modifier,
        topBar = {
            CollectionScreenTopBar(title = "Collections", onBack = onNavigateBack)
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
            CollectionsContent(
                state = state,
                rootCollections = rootCollections,
                gridState = gridState,
                onRetry = { viewModel.load() },
                onNavigateToCollection = onNavigateToCollection,
            )
        }
    }
}

@Composable
private fun CollectionsContent(
    state: CollectionsState,
    rootCollections: List<CollectionResponse>,
    gridState: LazyGridState,
    onRetry: () -> Unit,
    onNavigateToCollection: (String) -> Unit,
) {
    when {
        state.isLoading && state.collections.isEmpty() -> {
            Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                CircularProgressIndicator()
            }
        }

        state.error != null && state.collections.isEmpty() -> {
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
                TextButton(onClick = onRetry) { Text("Retry") }
            }
        }

        rootCollections.isEmpty() -> {
            Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                Text(
                    text = "No collections yet",
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
        }

        else -> {
            val collectionCountLabel =
                "${rootCollections.size} collection${if (rootCollections.size == 1) "" else "s"}"
            LazyVerticalGrid(
                columns = GridCells.Fixed(2),
                state = gridState,
                contentPadding = PaddingValues(IndelibleSpacing.step16),
                horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step12),
                verticalArrangement = Arrangement.spacedBy(IndelibleSpacing.step12),
                modifier = Modifier.fillMaxSize(),
            ) {
                item(span = { GridItemSpan(maxLineSpan) }) {
                    Text(
                        text = collectionCountLabel,
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                        modifier = Modifier.padding(vertical = IndelibleSpacing.step8),
                    )
                }
                items(
                    items = rootCollections,
                    key = { it.id },
                ) { collection ->
                    CollectionCard(
                        collection = collection,
                        onClick = { onNavigateToCollection(collection.id) },
                    )
                }
                if (state.isLoadingMore) {
                    item(span = { GridItemSpan(maxLineSpan) }) {
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
    }
}

@Composable
private fun CollectionCard(
    collection: CollectionResponse,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val banners = IndelibleTheme.colors.collectionBanners
    val bannerColor = banners[collectionBannerIndex(collection.color, collection.id)]

    Card(
        onClick = onClick,
        modifier = modifier.fillMaxWidth(),
        shape = MaterialTheme.shapes.extraLarge,
        colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surface),
        elevation = CardDefaults.cardElevation(defaultElevation = IndelibleSpacing.step2),
    ) {
        Column {
            // Colour banner
            Box(
                modifier =
                    Modifier
                        .fillMaxWidth()
                        .height(IndelibleSpacing.step64)
                        .background(bannerColor),
                contentAlignment = Alignment.Center,
            ) {
                val emoji = collection.icon?.takeIf { it.isNotBlank() }
                if (emoji != null) {
                    Text(
                        text = emoji,
                        style = MaterialTheme.typography.headlineMedium,
                    )
                } else {
                    Icon(
                        imageVector = IndelibleIcons.Folder,
                        contentDescription = null,
                        tint = MaterialTheme.colorScheme.onSurfaceVariant,
                        modifier = Modifier.size(IndelibleSpacing.step32),
                    )
                }
            }

            // Body
            Column(
                modifier =
                    Modifier.padding(
                        horizontal = IndelibleSpacing.step12,
                        vertical = IndelibleSpacing.step10,
                    ),
                verticalArrangement = Arrangement.spacedBy(IndelibleSpacing.step4),
            ) {
                Text(
                    text = collection.name,
                    style = MaterialTheme.typography.titleSmall,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                    color = MaterialTheme.colorScheme.onSurface,
                )
                val description = collection.description
                if (!description.isNullOrBlank()) {
                    Text(
                        text = description,
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                        maxLines = 2,
                        overflow = TextOverflow.Ellipsis,
                    )
                }
                Text(
                    text = "${collection.itemCount} item${if (collection.itemCount == 1L) "" else "s"}",
                    style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
        }
    }
}
