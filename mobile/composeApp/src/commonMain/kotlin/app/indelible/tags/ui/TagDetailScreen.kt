package app.indelible.tags.ui

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
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.rememberScrollState
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.automirrored.filled.ArrowForwardIos
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
import androidx.compose.runtime.remember
import androidx.compose.runtime.snapshotFlow
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import app.indelible.core.i18n.LocaleFormatters
import app.indelible.core.i18n.resolve
import app.indelible.library.ui.components.LibraryItemRow
import app.indelible.reader.model.TagData
import app.indelible.tags.viewmodel.TagDetailViewModel
import app.indelible.ui.theme.IndelibleIcons
import app.indelible.ui.theme.IndelibleShape
import app.indelible.ui.theme.IndelibleSpacing
import app.indelible.ui.theme.IndelibleTheme
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.common_back
import indelible.composeapp.generated.resources.common_retry
import indelible.composeapp.generated.resources.tags_items
import indelible.composeapp.generated.resources.tags_no_items
import indelible.composeapp.generated.resources.tags_stats
import indelible.composeapp.generated.resources.tags_title
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.map
import org.jetbrains.compose.resources.stringResource

private const val ITEMS_PAGINATION_TRIGGER = 5

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun TagDetailScreen(
    viewModel: TagDetailViewModel,
    onNavigateBack: () -> Unit,
    onNavigateToTag: (String) -> Unit,
    onNavigateToItem: (String) -> Unit,
    modifier: Modifier = Modifier,
) {
    val state by viewModel.state.collectAsState()
    val listState = rememberLazyListState()
    val rolledUpItemCounts = remember(state.allTags) { computeRolledUpItemCounts(state.allTags) }

    LaunchedEffect(listState) {
        snapshotFlow { listState.layoutInfo }
            .map { info ->
                val lastVisible = info.visibleItemsInfo.lastOrNull()?.index ?: 0
                val total = info.totalItemsCount
                total > 0 && lastVisible >= total - ITEMS_PAGINATION_TRIGGER
            }.distinctUntilChanged()
            .collect { nearEnd ->
                if (nearEnd) viewModel.loadNextItemsPage()
            }
    }

    Scaffold(
        modifier = modifier,
        topBar = {
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
                        contentDescription = stringResource(Res.string.common_back),
                        tint = MaterialTheme.colorScheme.primary,
                    )
                }
                Text(
                    text = stringResource(Res.string.tags_title),
                    style = MaterialTheme.typography.titleLarge,
                    modifier = Modifier.weight(1f),
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
            }
        },
    ) { paddingValues ->
        when {
            state.isLoading && state.tag == null -> {
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

            state.error != null && state.tag == null -> {
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
                    TagDetailContent(
                        state = state,
                        listState = listState,
                        rolledUpItemCounts = rolledUpItemCounts,
                        onNavigateToTag = onNavigateToTag,
                        onNavigateToItem = onNavigateToItem,
                    )
                }
            }
        }
    }
}

@Composable
private fun TagDetailContent(
    state: app.indelible.tags.viewmodel.TagDetailState,
    listState: LazyListState,
    rolledUpItemCounts: Map<String, Long>,
    onNavigateToTag: (String) -> Unit,
    onNavigateToItem: (String) -> Unit,
) {
    LazyColumn(state = listState, modifier = Modifier.fillMaxSize()) {
        state.tag?.let { tag ->
            item(key = "header") {
                TagHeroHeader(tag = tag, childCount = state.children.size)
            }
        }

        if (state.children.isNotEmpty()) {
            item(key = "children") {
                SubTagStrip(
                    children = state.children,
                    rolledUpItemCounts = rolledUpItemCounts,
                    onNavigate = onNavigateToTag,
                )
            }
        }

        if (state.items.isNotEmpty() || state.isLoadingMoreItems) {
            item(key = "items-header") {
                Text(
                    text = stringResource(Res.string.tags_items),
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

        if (state.items.isEmpty() && !state.isLoading && state.tag != null) {
            item(key = "empty-items") {
                Box(
                    modifier =
                        Modifier
                            .fillMaxWidth()
                            .padding(vertical = IndelibleSpacing.step40),
                    contentAlignment = Alignment.Center,
                ) {
                    Text(
                        text = stringResource(Res.string.tags_no_items),
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
            }
        }

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
private fun TagHeroHeader(
    tag: TagData,
    childCount: Int,
) {
    val tagColors = IndelibleTheme.colors.tagColors
    val accentColor = tagColors[tagColorIndex(tag.color, tag.id)]
    val bannerBg = accentColor.copy(alpha = 0.07f)
    val iconBg = accentColor.copy(alpha = 0.13f)

    Column {
        Box(
            modifier =
                Modifier
                    .fillMaxWidth()
                    .height(IndelibleSpacing.step80)
                    .background(bannerBg),
            contentAlignment = Alignment.Center,
        ) {
            Box(
                modifier =
                    Modifier
                        .size(IndelibleSpacing.step48)
                        .clip(MaterialTheme.shapes.extraLarge)
                        .background(iconBg),
                contentAlignment = Alignment.Center,
            ) {
                Icon(
                    imageVector = IndelibleIcons.Tag,
                    contentDescription = null,
                    tint = accentColor,
                    modifier = Modifier.size(IndelibleSpacing.step24),
                )
            }
        }

        Column(
            modifier =
                Modifier.padding(
                    horizontal = IndelibleSpacing.step16,
                    vertical = IndelibleSpacing.step12,
                ),
            verticalArrangement = Arrangement.spacedBy(IndelibleSpacing.step4),
        ) {
            Text(
                text = "#${tag.name}",
                style = MaterialTheme.typography.headlineSmall,
                fontWeight = FontWeight.Bold,
                color = MaterialTheme.colorScheme.onSurface,
            )
            val statsText =
                stringResource(
                    Res.string.tags_stats,
                    tag.itemCount,
                    tag.highlightCount,
                    childCount,
                )
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
private fun SubTagStrip(
    children: List<TagData>,
    rolledUpItemCounts: Map<String, Long>,
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
                SubTagChip(
                    tag = child,
                    rolledUpItemCount = rolledUpItemCounts[child.id] ?: child.itemCount,
                    onClick = { onNavigate(child.id) },
                )
            }
        }
        HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant)
    }
}

@Composable
private fun SubTagChip(
    tag: TagData,
    rolledUpItemCount: Long,
    onClick: () -> Unit,
) {
    val tagColors = IndelibleTheme.colors.tagColors
    val dotColor = tagColors[tagColorIndex(tag.color, tag.id)]

    Row(
        modifier =
            Modifier
                .clip(IndelibleShape.lg)
                .background(MaterialTheme.colorScheme.surfaceVariant)
                .border(
                    width = IndelibleSpacing.step2 / 2,
                    color = MaterialTheme.colorScheme.outlineVariant,
                    shape = IndelibleShape.lg,
                ).clickable(onClick = onClick)
                .padding(
                    horizontal = IndelibleSpacing.step10,
                    vertical = IndelibleSpacing.step6,
                ),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step6),
    ) {
        Box(
            modifier =
                Modifier
                    .size(IndelibleSpacing.step8)
                    .clip(MaterialTheme.shapes.extraLarge)
                    .background(dotColor),
        )
        Text(
            text = tag.name,
            style = MaterialTheme.typography.labelSmall,
            color = MaterialTheme.colorScheme.onSurface,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
        Text(
            text = LocaleFormatters.number(rolledUpItemCount),
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
