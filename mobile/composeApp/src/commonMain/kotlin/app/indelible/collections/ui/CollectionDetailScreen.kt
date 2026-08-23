package app.indelible.collections.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.horizontalScroll
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
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowForwardIos
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.HorizontalDivider
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
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import app.indelible.api.generated.models.CollectionResponse
import app.indelible.collections.ui.components.CollectionScreenTopBar
import app.indelible.collections.ui.components.PaginationEffect
import app.indelible.collections.viewmodel.CollectionDetailState
import app.indelible.collections.viewmodel.CollectionDetailViewModel
import app.indelible.core.i18n.resolve
import app.indelible.library.ui.components.LibraryItemRow
import app.indelible.ui.theme.IndelibleIcons
import app.indelible.ui.theme.IndelibleSpacing
import app.indelible.ui.theme.IndelibleTheme
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.collections_item_count
import indelible.composeapp.generated.resources.collections_items
import indelible.composeapp.generated.resources.collections_no_items
import indelible.composeapp.generated.resources.collections_stats
import indelible.composeapp.generated.resources.collections_untitled
import indelible.composeapp.generated.resources.common_retry
import org.jetbrains.compose.resources.pluralStringResource
import org.jetbrains.compose.resources.stringResource

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun CollectionDetailScreen(
    viewModel: CollectionDetailViewModel,
    onNavigateBack: () -> Unit,
    onNavigateToCollection: (String) -> Unit,
    onNavigateToItem: (String) -> Unit,
    modifier: Modifier = Modifier,
) {
    val state by viewModel.state.collectAsState()
    val listState = rememberLazyListState()

    PaginationEffect(
        listState = listState,
        hasMore = state.hasMoreItems,
        onLoadMore = { viewModel.loadNextItemsPage() },
    )

    Scaffold(
        modifier = modifier,
        topBar = {
            CollectionScreenTopBar(
                title = state.collection?.name ?: stringResource(Res.string.collections_untitled),
                onBack = onNavigateBack,
            )
        },
    ) { paddingValues ->
        when {
            state.isLoading && state.collection == null -> {
                Box(
                    modifier =
                        Modifier
                            .fillMaxSize()
                            .padding(paddingValues),
                    contentAlignment = Alignment.Center,
                ) {
                    CircularProgressIndicator()
                }
            }

            state.error != null && state.collection == null -> {
                Column(
                    modifier =
                        Modifier
                            .fillMaxSize()
                            .padding(paddingValues),
                    horizontalAlignment = Alignment.CenterHorizontally,
                    verticalArrangement = Arrangement.Center,
                ) {
                    Text(
                        text = state.error?.resolve().orEmpty(),
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.error,
                    )
                    Spacer(modifier = Modifier.height(IndelibleSpacing.step12))
                    TextButton(onClick = { viewModel.load() }) {
                        Text(stringResource(Res.string.common_retry))
                    }
                }
            }

            else -> {
                PullToRefreshBox(
                    isRefreshing = state.isRefreshing,
                    onRefresh = { viewModel.refresh() },
                    modifier =
                        Modifier
                            .fillMaxSize()
                            .padding(paddingValues),
                ) {
                    CollectionDetailContent(
                        state = state,
                        listState = listState,
                        onNavigateToCollection = onNavigateToCollection,
                        onNavigateToItem = onNavigateToItem,
                    )
                }
            }
        }
    }
}

@Composable
private fun CollectionDetailContent(
    state: CollectionDetailState,
    listState: LazyListState,
    onNavigateToCollection: (String) -> Unit,
    onNavigateToItem: (String) -> Unit,
) {
    LazyColumn(state = listState, modifier = Modifier.fillMaxSize()) {
        // Collection hero header
        state.collection?.let { col ->
            item(key = "header") {
                CollectionHeroHeader(collection = col, childCount = state.children.size)
            }
        }

        // Sub-collections strip
        if (state.children.isNotEmpty()) {
            item(key = "children") {
                SubCollectionStrip(
                    children = state.children,
                    onNavigate = onNavigateToCollection,
                )
            }
        }

        // Items section header
        if (state.items.isNotEmpty() || state.isLoadingMoreItems) {
            item(key = "items-header") {
                Text(
                    text = stringResource(Res.string.collections_items),
                    style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    modifier =
                        Modifier.padding(
                            start = IndelibleSpacing.step16,
                            end = IndelibleSpacing.step16,
                            top = IndelibleSpacing.step16,
                            bottom = IndelibleSpacing.step4,
                        ),
                )
            }
        }

        items(items = state.items, key = { it.id }) { item ->
            LibraryItemRow(
                item = item,
                onClick = { onNavigateToItem(item.documentId) },
            )
        }

        // Empty items state
        if (state.items.isEmpty() && !state.isLoading && state.collection != null) {
            item(key = "empty-items") {
                Box(
                    modifier =
                        Modifier
                            .fillMaxWidth()
                            .padding(vertical = IndelibleSpacing.step40),
                    contentAlignment = Alignment.Center,
                ) {
                    Text(
                        text = stringResource(Res.string.collections_no_items),
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
            }
        }

        // Loading more indicator
        if (state.isLoadingMoreItems) {
            item(key = "loading-more") {
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

        item(key = "bottom-spacer") {
            Spacer(modifier = Modifier.height(IndelibleSpacing.step24))
        }
    }
}

@Composable
private fun CollectionHeroHeader(
    collection: CollectionResponse,
    childCount: Int,
) {
    val banners = IndelibleTheme.colors.collectionBanners
    val bannerColor = banners[collectionBannerIndex(collection.color, collection.id)]

    Column {
        // Colored banner with emoji/icon
        Box(
            modifier =
                Modifier
                    .fillMaxWidth()
                    .height(IndelibleSpacing.step80)
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
                    modifier = Modifier.size(IndelibleSpacing.step40),
                )
            }
        }

        // Name, description, stats
        Column(
            modifier =
                Modifier.padding(
                    horizontal = IndelibleSpacing.step16,
                    vertical = IndelibleSpacing.step12,
                ),
            verticalArrangement = Arrangement.spacedBy(IndelibleSpacing.step4),
        ) {
            Text(
                text = collection.name,
                style = MaterialTheme.typography.titleLarge,
                fontWeight = FontWeight.Bold,
                color = MaterialTheme.colorScheme.onSurface,
            )
            if (!collection.description.isNullOrBlank()) {
                Text(
                    text = collection.description,
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    maxLines = 3,
                    overflow = TextOverflow.Ellipsis,
                )
            }
            val statsText =
                if (childCount > 0) {
                    stringResource(Res.string.collections_stats, collection.itemCount, childCount)
                } else {
                    pluralStringResource(
                        Res.plurals.collections_item_count,
                        collection.itemCount.toInt(),
                        collection.itemCount,
                    )
                }
            Text(
                text = statsText,
                style = MaterialTheme.typography.labelSmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }

        HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant)
    }
}

@Composable
private fun SubCollectionStrip(
    children: List<CollectionResponse>,
    onNavigate: (String) -> Unit,
) {
    Column {
        Row(
            modifier =
                Modifier
                    .horizontalScroll(rememberScrollState())
                    .padding(
                        horizontal = IndelibleSpacing.step16,
                        vertical = IndelibleSpacing.step12,
                    ),
            horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step8),
        ) {
            children.forEach { child ->
                SubCollectionChip(collection = child, onClick = { onNavigate(child.id) })
            }
        }
        HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant)
    }
}

@Composable
private fun SubCollectionChip(
    collection: CollectionResponse,
    onClick: () -> Unit,
) {
    Row(
        modifier =
            Modifier
                .clip(RoundedCornerShape(IndelibleSpacing.step12))
                .background(MaterialTheme.colorScheme.surfaceVariant)
                .border(
                    width = IndelibleSpacing.step2 / 2,
                    color = MaterialTheme.colorScheme.outlineVariant,
                    shape = RoundedCornerShape(IndelibleSpacing.step12),
                ).clickable(onClick = onClick)
                .padding(
                    horizontal = IndelibleSpacing.step10,
                    vertical = IndelibleSpacing.step6,
                ),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step6),
    ) {
        val emoji = collection.icon?.takeIf { it.isNotBlank() }
        if (emoji != null) {
            Text(text = emoji, style = MaterialTheme.typography.labelSmall)
        } else {
            Icon(
                imageVector = IndelibleIcons.Folder,
                contentDescription = null,
                tint = MaterialTheme.colorScheme.onSurfaceVariant,
                modifier = Modifier.size(IndelibleSpacing.step14),
            )
        }
        Text(
            text = collection.name,
            style = MaterialTheme.typography.labelSmall,
            color = MaterialTheme.colorScheme.onSurface,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
        Text(
            text = "${collection.itemCount}",
            style = MaterialTheme.typography.labelSmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
        Icon(
            imageVector = Icons.AutoMirrored.Filled.ArrowForwardIos,
            contentDescription = null,
            tint = MaterialTheme.colorScheme.onSurfaceVariant,
            modifier = Modifier.size(IndelibleSpacing.step12),
        )
    }
}
