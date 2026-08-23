package app.indelible.reader.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Close
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.Text
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.unit.dp
import app.indelible.core.i18n.LocaleFormatters
import app.indelible.reader.model.TagData
import app.indelible.ui.theme.IndelibleIcons
import app.indelible.ui.theme.IndelibleSpacing
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.reader_action_remove_tag
import indelible.composeapp.generated.resources.reader_create_tag
import indelible.composeapp.generated.resources.reader_tags
import indelible.composeapp.generated.resources.reader_tags_empty
import org.jetbrains.compose.resources.stringResource

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun HighlightTagSheet(
    appliedTags: List<String>,
    availableTags: List<TagData>,
    onTagsChanged: (List<String>) -> Unit,
    onDismiss: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true)
    var currentTags by remember { mutableStateOf(appliedTags) }
    var searchQuery by remember { mutableStateOf("") }
    val focusRequester = remember { FocusRequester() }

    val filteredSuggestions =
        remember(availableTags, searchQuery, currentTags) {
            val q = searchQuery.trim().lowercase()
            availableTags.filter { tag ->
                tag.name !in currentTags &&
                    (q.isEmpty() || tag.name.lowercase().contains(q))
            }
        }

    val canCreateNew =
        remember(searchQuery, currentTags, availableTags) {
            val trimmed = searchQuery.trim()
            trimmed.isNotEmpty() &&
                trimmed !in currentTags &&
                availableTags.none { it.name.equals(trimmed, ignoreCase = true) }
        }

    ModalBottomSheet(
        onDismissRequest = {
            onTagsChanged(currentTags)
            onDismiss()
        },
        sheetState = sheetState,
        modifier = modifier,
    ) {
        HighlightTagSheetContent(
            currentTags = currentTags,
            filteredSuggestions = filteredSuggestions,
            searchQuery = searchQuery,
            canCreateNew = canCreateNew,
            focusRequester = focusRequester,
            onSearchQueryChanged = { searchQuery = it },
            onAddTag = { name ->
                if (name !in currentTags) {
                    currentTags = currentTags + name
                    onTagsChanged(currentTags)
                }
                searchQuery = ""
            },
            onRemoveTag = { name ->
                currentTags = currentTags - name
                onTagsChanged(currentTags)
            },
        )
    }

    LaunchedEffect(Unit) {
        focusRequester.requestFocus()
    }
}

@OptIn(ExperimentalLayoutApi::class)
@Composable
private fun HighlightTagSheetContent(
    currentTags: List<String>,
    filteredSuggestions: List<TagData>,
    searchQuery: String,
    canCreateNew: Boolean,
    focusRequester: FocusRequester,
    onSearchQueryChanged: (String) -> Unit,
    onAddTag: (String) -> Unit,
    onRemoveTag: (String) -> Unit,
    modifier: Modifier = Modifier,
) {
    Column(
        modifier =
            modifier
                .fillMaxWidth()
                .padding(
                    horizontal = IndelibleSpacing.screenPaddingH,
                    vertical = IndelibleSpacing.step16,
                ),
    ) {
        Text(
            text = stringResource(Res.string.reader_tags),
            style = MaterialTheme.typography.titleMedium,
            color = MaterialTheme.colorScheme.onSurface,
        )

        Spacer(modifier = Modifier.height(IndelibleSpacing.step16))

        if (currentTags.isNotEmpty()) {
            FlowRow(
                horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step8),
                verticalArrangement = Arrangement.spacedBy(IndelibleSpacing.step8),
                modifier = Modifier.fillMaxWidth(),
            ) {
                currentTags.forEach { tag ->
                    AppliedTagChip(
                        name = tag,
                        onRemove = { onRemoveTag(tag) },
                    )
                }
            }
            Spacer(modifier = Modifier.height(IndelibleSpacing.step12))
        }

        TagSearchField(
            query = searchQuery,
            onQueryChange = onSearchQueryChanged,
            onDone = { trimmed -> onAddTag(trimmed) },
            containerShape = RoundedCornerShape(8.dp),
            iconSize = 18.dp,
            focusRequester = focusRequester,
        )

        Spacer(modifier = Modifier.height(IndelibleSpacing.step8))

        LazyColumn(modifier = Modifier.widthIn(max = 500.dp)) {
            if (canCreateNew) {
                item {
                    TagSuggestionRow(
                        name = searchQuery.trim(),
                        count = null,
                        isCreate = true,
                        onClick = { onAddTag(searchQuery.trim()) },
                    )
                }
            }

            if ((canCreateNew || filteredSuggestions.isNotEmpty()) && filteredSuggestions.isNotEmpty()) {
                item {
                    HorizontalDivider(
                        color = MaterialTheme.colorScheme.outlineVariant,
                        modifier = Modifier.padding(vertical = IndelibleSpacing.step4),
                    )
                }
            }

            items(filteredSuggestions, key = { it.id }) { tag ->
                TagSuggestionRow(
                    name = tag.name,
                    count = tag.highlightCount.toInt(),
                    isCreate = false,
                    onClick = { onAddTag(tag.name) },
                )
            }

            if (filteredSuggestions.isEmpty() && !canCreateNew && searchQuery.isEmpty()) {
                item {
                    Text(
                        text = stringResource(Res.string.reader_tags_empty),
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                        modifier = Modifier.padding(vertical = IndelibleSpacing.step16),
                    )
                }
            }
        }

        Spacer(modifier = Modifier.height(IndelibleSpacing.step16))
    }
}

@Composable
private fun AppliedTagChip(
    name: String,
    onRemove: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Row(
        modifier =
            modifier
                .clip(RoundedCornerShape(16.dp))
                .background(MaterialTheme.colorScheme.secondaryContainer)
                .padding(
                    start = IndelibleSpacing.step10,
                    top = IndelibleSpacing.step4,
                    end = IndelibleSpacing.step4,
                    bottom = IndelibleSpacing.step4,
                ),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(4.dp),
    ) {
        Text(
            text = name,
            style = MaterialTheme.typography.labelMedium,
            color = MaterialTheme.colorScheme.onSecondaryContainer,
        )
        Box(
            modifier =
                Modifier
                    .size(20.dp)
                    .clip(CircleShape)
                    .clickable(onClick = onRemove),
            contentAlignment = Alignment.Center,
        ) {
            Icon(
                Icons.Filled.Close,
                contentDescription = stringResource(Res.string.reader_action_remove_tag),
                tint = MaterialTheme.colorScheme.onSecondaryContainer,
                modifier = Modifier.size(14.dp),
            )
        }
    }
}

@Composable
private fun TagSuggestionRow(
    name: String,
    count: Int?,
    isCreate: Boolean,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Row(
        modifier =
            modifier
                .fillMaxWidth()
                .clickable(onClick = onClick)
                .padding(vertical = IndelibleSpacing.step10),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step12),
    ) {
        Icon(
            IndelibleIcons.Tag,
            contentDescription = null,
            tint = MaterialTheme.colorScheme.onSurfaceVariant,
            modifier = Modifier.size(18.dp),
        )
        Text(
            text = if (isCreate) stringResource(Res.string.reader_create_tag, name) else name,
            style = MaterialTheme.typography.bodyMedium,
            color = if (isCreate) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onSurface,
            modifier = Modifier.weight(1f),
        )
        if (!isCreate && count != null) {
            Text(
                text = LocaleFormatters.number(count.toLong()),
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
    }
}
