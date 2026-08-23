package app.indelible.tags.ui

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
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
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.automirrored.filled.KeyboardArrowRight
import androidx.compose.material.icons.filled.Close
import androidx.compose.material.icons.filled.Search
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
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import app.indelible.core.i18n.resolve
import app.indelible.reader.model.TagData
import app.indelible.tags.viewmodel.TagScope
import app.indelible.tags.viewmodel.TagsState
import app.indelible.tags.viewmodel.TagsViewModel
import app.indelible.ui.theme.IndelibleShape
import app.indelible.ui.theme.IndelibleSpacing
import app.indelible.ui.theme.IndelibleTheme
import app.indelible.ui.theme.paletteBucketIndex
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.common_back
import indelible.composeapp.generated.resources.common_retry
import indelible.composeapp.generated.resources.tags_clear_filter_cd
import indelible.composeapp.generated.resources.tags_filter_placeholder
import indelible.composeapp.generated.resources.tags_highlight_count
import indelible.composeapp.generated.resources.tags_item_count
import indelible.composeapp.generated.resources.tags_no_match
import indelible.composeapp.generated.resources.tags_none
import indelible.composeapp.generated.resources.tags_scope_document
import indelible.composeapp.generated.resources.tags_scope_highlight
import indelible.composeapp.generated.resources.tags_title
import org.jetbrains.compose.resources.pluralStringResource
import org.jetbrains.compose.resources.stringResource

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun TagsScreen(
    viewModel: TagsViewModel,
    onNavigateBack: () -> Unit,
    onNavigateToTag: (String) -> Unit,
    modifier: Modifier = Modifier,
) {
    val state by viewModel.state.collectAsState()

    val rolledUpItemCounts = remember(state.tags) { computeRolledUpItemCounts(state.tags) }

    // Match web root logic: show a tag if it has no parent, or its parent is not
    // in the returned set (e.g. scope=document returns child B but not parent A → B is a root).
    val tagIds = state.tags.map { it.id }.toSet()
    val visibleTags = state.tags.filter { it.parentId == null || !tagIds.contains(it.parentId) }
    val filteredTags =
        if (state.filter.isBlank()) {
            visibleTags
        } else {
            visibleTags.filter { it.name.contains(state.filter, ignoreCase = true) }
        }

    Scaffold(
        modifier = modifier,
        topBar = {
            TagsTopBar(
                scope = state.scope,
                onNavigateBack = onNavigateBack,
                onScopeToggle = { viewModel.toggleScope(it) },
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
                state.isLoading && state.tags.isEmpty() -> {
                    Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                        CircularProgressIndicator()
                    }
                }

                state.error != null && state.tags.isEmpty() -> {
                    Column(
                        modifier = Modifier.fillMaxSize(),
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
                    TagsContent(
                        state = state,
                        filteredTags = filteredTags,
                        rolledUpItemCounts = rolledUpItemCounts,
                        onNavigateToTag = onNavigateToTag,
                        onValueChange = { viewModel.setFilter(it) },
                    )
                }
            }
        }
    }
}

@Composable
private fun TagsTopBar(
    scope: TagScope,
    onNavigateBack: () -> Unit,
    onScopeToggle: (TagScope) -> Unit,
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
                contentDescription = stringResource(Res.string.common_back),
                tint = MaterialTheme.colorScheme.primary,
            )
        }
        Text(
            text = stringResource(Res.string.tags_title),
            style = MaterialTheme.typography.titleLarge,
            modifier = Modifier.weight(1f),
        )
        TagScopeSegmentedControl(
            scope = scope,
            onScopeToggle = onScopeToggle,
            modifier = Modifier.padding(end = IndelibleSpacing.step12),
        )
    }
}

@Composable
private fun TagsContent(
    state: TagsState,
    filteredTags: List<TagData>,
    rolledUpItemCounts: Map<String, Long>,
    onNavigateToTag: (String) -> Unit,
    onValueChange: (String) -> Unit,
) {
    Column(modifier = Modifier.fillMaxSize()) {
        TagFilterBar(
            value = state.filter,
            onValueChange = onValueChange,
            modifier =
                Modifier
                    .fillMaxWidth()
                    .padding(
                        horizontal = IndelibleSpacing.step16,
                        vertical = IndelibleSpacing.step8,
                    ),
        )

        when {
            filteredTags.isEmpty() && state.filter.isNotBlank() -> {
                Box(
                    modifier = Modifier.fillMaxSize(),
                    contentAlignment = Alignment.Center,
                ) {
                    Text(
                        text = stringResource(Res.string.tags_no_match, state.filter),
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
            }

            filteredTags.isEmpty() -> {
                Box(
                    modifier = Modifier.fillMaxSize(),
                    contentAlignment = Alignment.Center,
                ) {
                    Text(
                        text = stringResource(Res.string.tags_none),
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
            }

            else -> {
                LazyColumn(modifier = Modifier.fillMaxSize()) {
                    items(items = filteredTags, key = { it.id }) { tag ->
                        TagRow(
                            tag = tag,
                            scope = state.scope,
                            rolledUpItemCount = rolledUpItemCounts[tag.id] ?: tag.itemCount,
                            onClick = { onNavigateToTag(tag.id) },
                        )
                    }
                }
            }
        }
    }
}

@Composable
internal fun TagScopeSegmentedControl(
    scope: TagScope,
    onScopeToggle: (TagScope) -> Unit,
    modifier: Modifier = Modifier,
) {
    Row(
        modifier =
            modifier
                .clip(MaterialTheme.shapes.small)
                .background(MaterialTheme.colorScheme.surfaceContainerHigh)
                .padding(IndelibleSpacing.step2),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        TagScopeItem(
            label = stringResource(Res.string.tags_scope_document),
            selected = scope == TagScope.DOC,
            onClick = { onScopeToggle(TagScope.DOC) },
        )
        TagScopeItem(
            label = stringResource(Res.string.tags_scope_highlight),
            selected = scope == TagScope.HIGHLIGHT,
            onClick = { onScopeToggle(TagScope.HIGHLIGHT) },
        )
    }
}

@Composable
private fun TagScopeItem(
    label: String,
    selected: Boolean,
    onClick: () -> Unit,
) {
    val bgColor =
        if (selected) {
            MaterialTheme.colorScheme.surfaceContainer
        } else {
            androidx.compose.ui.graphics.Color.Transparent
        }
    Box(
        modifier =
            Modifier
                .clip(IndelibleShape.xs)
                .background(bgColor)
                .clickable(onClick = onClick)
                .padding(horizontal = IndelibleSpacing.step10, vertical = IndelibleSpacing.step4),
        contentAlignment = Alignment.Center,
    ) {
        Text(
            text = label,
            style = MaterialTheme.typography.labelSmall,
            fontWeight = if (selected) FontWeight.SemiBold else FontWeight.Medium,
            color =
                if (selected) {
                    MaterialTheme.colorScheme.onSurface
                } else {
                    MaterialTheme.colorScheme.onSurfaceVariant
                },
        )
    }
}

@Composable
private fun TagFilterBar(
    value: String,
    onValueChange: (String) -> Unit,
    modifier: Modifier = Modifier,
) {
    Row(
        modifier =
            modifier
                .clip(IndelibleShape.md)
                .background(MaterialTheme.colorScheme.surfaceVariant)
                .padding(horizontal = IndelibleSpacing.step12, vertical = IndelibleSpacing.step8),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Icon(
            imageVector = Icons.Filled.Search,
            contentDescription = null,
            tint = MaterialTheme.colorScheme.onSurfaceVariant,
            modifier = Modifier.size(IndelibleSpacing.step16),
        )
        Spacer(modifier = Modifier.width(IndelibleSpacing.step8))
        BasicTextField(
            value = value,
            onValueChange = onValueChange,
            modifier = Modifier.weight(1f),
            textStyle =
                MaterialTheme.typography.bodyLarge.copy(
                    color = MaterialTheme.colorScheme.onSurface,
                ),
            singleLine = true,
            cursorBrush = SolidColor(MaterialTheme.colorScheme.primary),
            keyboardOptions = KeyboardOptions(imeAction = ImeAction.Done),
            decorationBox = { innerTextField ->
                if (value.isEmpty()) {
                    Text(
                        text = stringResource(Res.string.tags_filter_placeholder),
                        style = MaterialTheme.typography.bodyLarge,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
                innerTextField()
            },
        )
        AnimatedVisibility(visible = value.isNotEmpty(), enter = fadeIn(), exit = fadeOut()) {
            IconButton(
                onClick = { onValueChange("") },
                modifier = Modifier.size(IndelibleSpacing.step20),
            ) {
                Icon(
                    imageVector = Icons.Filled.Close,
                    contentDescription = stringResource(Res.string.tags_clear_filter_cd),
                    tint = MaterialTheme.colorScheme.onSurfaceVariant,
                    modifier = Modifier.size(IndelibleSpacing.step16),
                )
            }
        }
    }
}

@Composable
internal fun TagRow(
    tag: TagData,
    scope: TagScope,
    rolledUpItemCount: Long,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val dotColor = IndelibleTheme.colors.tagColors[tagColorIndex(tag.color, tag.id)]
    // Matches web getTagCount: highlight uses direct count, doc/all use rolled-up item count.
    val (count, countResource) =
        when (scope) {
            TagScope.HIGHLIGHT -> tag.highlightCount to Res.plurals.tags_highlight_count
            TagScope.DOC -> rolledUpItemCount to Res.plurals.tags_item_count
            TagScope.ALL -> (rolledUpItemCount + tag.highlightCount) to Res.plurals.tags_item_count
        }

    Column(modifier = modifier.background(MaterialTheme.colorScheme.surface)) {
        Row(
            modifier =
                Modifier
                    .fillMaxWidth()
                    .clickable(onClick = onClick)
                    .padding(
                        horizontal = IndelibleSpacing.rowPaddingH,
                        vertical = IndelibleSpacing.rowPaddingV,
                    ),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Box(
                modifier =
                    Modifier
                        .size(IndelibleSpacing.step8)
                        .clip(MaterialTheme.shapes.extraLarge)
                        .background(dotColor),
            )
            Spacer(modifier = Modifier.width(IndelibleSpacing.step12))
            Text(
                text = tag.name,
                style = MaterialTheme.typography.titleSmall,
                color = MaterialTheme.colorScheme.onSurface,
                modifier = Modifier.weight(1f),
            )
            Text(
                text = pluralStringResource(countResource, count.toInt(), count),
                style = MaterialTheme.typography.labelSmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
            Spacer(modifier = Modifier.width(IndelibleSpacing.step8))
            Icon(
                imageVector = Icons.AutoMirrored.Filled.KeyboardArrowRight,
                contentDescription = null,
                tint = MaterialTheme.colorScheme.onSurfaceVariant,
                modifier = Modifier.size(IndelibleSpacing.step20),
            )
        }
        HorizontalDivider(
            color = MaterialTheme.colorScheme.outlineVariant,
            modifier = Modifier.padding(start = IndelibleSpacing.rowPaddingH),
        )
    }
}

/**
 * Recursively sums own item count + all descendants' item counts, matching the web's
 * rolledUpCount derivation. Cycle-safe via a visiting guard (same as web's SvelteSet guard).
 */
internal fun computeRolledUpItemCounts(tags: List<TagData>): Map<String, Long> {
    val childrenMap = mutableMapOf<String, MutableList<String>>()
    for (tag in tags) {
        val parentId = tag.parentId ?: continue
        childrenMap.getOrPut(parentId) { mutableListOf() }.add(tag.id)
    }
    val directCount = tags.associate { it.id to it.itemCount }
    val memo = mutableMapOf<String, Long>()
    val visiting = mutableSetOf<String>()

    fun sum(id: String): Long {
        memo[id]?.let { return it }
        if (id in visiting) return directCount[id] ?: 0L
        visiting.add(id)
        val total = (directCount[id] ?: 0L) + (childrenMap[id] ?: emptyList()).sumOf { sum(it) }
        memo[id] = total
        visiting.remove(id)
        return total
    }

    return tags.associate { it.id to sum(it.id) }
}

internal fun tagColorIndex(
    color: String?,
    id: String,
): Int = paletteBucketIndex(color, id)
