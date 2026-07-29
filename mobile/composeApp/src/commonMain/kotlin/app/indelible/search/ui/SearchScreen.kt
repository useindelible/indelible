package app.indelible.search.ui

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInVertically
import androidx.compose.animation.slideOutVertically
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.AlternateEmail
import androidx.compose.material.icons.filled.Close
import androidx.compose.material.icons.filled.Folder
import androidx.compose.material.icons.filled.History
import androidx.compose.material.icons.filled.Label
import androidx.compose.material.icons.filled.Person
import androidx.compose.material.icons.filled.Search
import androidx.compose.material.icons.filled.TipsAndUpdates
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.tooling.preview.Preview
import app.indelible.search.model.SearchResult
import app.indelible.search.model.SearchSuggestion
import app.indelible.search.ui.components.SearchResultRow
import app.indelible.search.viewmodel.SearchEffect
import app.indelible.search.viewmodel.SearchState
import app.indelible.search.viewmodel.SearchViewModel
import app.indelible.ui.theme.AppTheme
import app.indelible.ui.theme.IndelibleShape
import app.indelible.ui.theme.IndelibleSpacing
import kotlinx.datetime.Instant

@OptIn(ExperimentalMaterial3Api::class)
private const val SLIDE_OFFSET_DIVISOR = 3

@Composable
fun SearchScreen(
    viewModel: SearchViewModel,
    onNavigateToReader: (String) -> Unit,
    modifier: Modifier = Modifier,
) {
    val state by viewModel.state.collectAsState()
    val snackbarHostState = remember { SnackbarHostState() }
    val focusRequester = remember { FocusRequester() }

    LaunchedEffect(viewModel) {
        viewModel.effects.collect { effect ->
            when (effect) {
                is SearchEffect.NavigateToReader -> onNavigateToReader(effect.itemId)
                is SearchEffect.ShowSnackbar -> snackbarHostState.showSnackbar(effect.message)
            }
        }
    }

    Scaffold(
        modifier = modifier,
        snackbarHost = { SnackbarHost(snackbarHostState) },
    ) { paddingValues ->
        Column(
            modifier =
                Modifier
                    .fillMaxSize()
                    .padding(paddingValues),
        ) {
            SearchBarRow(
                query = state.query,
                onQueryChange = { viewModel.onQueryChange(it) },
                onSubmit = { viewModel.submitSearch() },
                onClear = { viewModel.clearQuery() },
                focusRequester = focusRequester,
            )
            HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant)

            Box(modifier = Modifier.fillMaxSize()) {
                if (state.submittedQuery.isNotBlank()) {
                    ResultsContent(
                        state = state,
                        onLoadMore = { viewModel.loadNextPage() },
                        onRefresh = { viewModel.refresh() },
                        onResultClick = { viewModel.onResultTap(it) },
                    )
                } else {
                    IdleContent(
                        state = state,
                        onRecentClick = { viewModel.selectRecentSearch(it) },
                        onRecentDelete = { viewModel.deleteRecentSearch(it) },
                        onClearAll = { viewModel.clearRecentSearches() },
                        onSyntaxPillClick = { viewModel.onQueryChange(state.query + it) },
                    )
                }

                SuggestionsOverlay(
                    visible = state.showSuggestions && state.suggestions.isNotEmpty(),
                    suggestions = state.suggestions,
                    onSelect = { viewModel.selectSuggestion(it) },
                    modifier =
                        Modifier
                            .align(Alignment.TopStart)
                            .fillMaxWidth(),
                )
            }
        }
    }
}

@Composable
private fun SearchBarRow(
    query: String,
    onQueryChange: (String) -> Unit,
    onSubmit: () -> Unit,
    onClear: () -> Unit,
    focusRequester: FocusRequester,
    modifier: Modifier = Modifier,
) {
    Row(
        modifier =
            modifier
                .fillMaxWidth()
                .padding(horizontal = IndelibleSpacing.step16, vertical = IndelibleSpacing.step8),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step8),
    ) {
        Row(
            modifier =
                Modifier
                    .weight(1f)
                    .clip(IndelibleShape.md)
                    .background(MaterialTheme.colorScheme.surfaceVariant)
                    .padding(horizontal = IndelibleSpacing.step12, vertical = IndelibleSpacing.step10),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Icon(
                imageVector = Icons.Filled.Search,
                contentDescription = null,
                modifier = Modifier.size(IndelibleSpacing.step20),
                tint = MaterialTheme.colorScheme.onSurfaceVariant,
            )
            Spacer(modifier = Modifier.width(IndelibleSpacing.step8))
            BasicTextField(
                value = query,
                onValueChange = onQueryChange,
                modifier =
                    Modifier
                        .weight(1f)
                        .focusRequester(focusRequester),
                textStyle =
                    MaterialTheme.typography.bodyLarge.copy(
                        color = MaterialTheme.colorScheme.onSurface,
                    ),
                singleLine = true,
                cursorBrush = SolidColor(MaterialTheme.colorScheme.primary),
                keyboardOptions = KeyboardOptions(imeAction = ImeAction.Search),
                keyboardActions = KeyboardActions(onSearch = { onSubmit() }),
                decorationBox = { innerTextField ->
                    if (query.isEmpty()) {
                        Text(
                            text = "Search your Library…",
                            style = MaterialTheme.typography.bodyLarge,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                    }
                    innerTextField()
                },
            )
            AnimatedVisibility(visible = query.isNotEmpty(), enter = fadeIn(), exit = fadeOut()) {
                IconButton(
                    onClick = onClear,
                    modifier = Modifier.size(IndelibleSpacing.step20),
                ) {
                    Icon(
                        imageVector = Icons.Filled.Close,
                        contentDescription = "Clear search",
                        modifier = Modifier.size(IndelibleSpacing.step16),
                        tint = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
            }
        }

        AnimatedVisibility(visible = query.isNotEmpty(), enter = fadeIn(), exit = fadeOut()) {
            TextButton(onClick = onClear) {
                Text(
                    text = "Cancel",
                    style = MaterialTheme.typography.bodyLarge,
                    color = MaterialTheme.colorScheme.primary,
                )
            }
        }
    }
}

@Composable
private fun SuggestionsOverlay(
    visible: Boolean,
    suggestions: List<SearchSuggestion>,
    onSelect: (insertText: String) -> Unit,
    modifier: Modifier = Modifier,
) {
    AnimatedVisibility(
        visible = visible,
        enter = fadeIn() + slideInVertically { -it / SLIDE_OFFSET_DIVISOR },
        exit = fadeOut() + slideOutVertically { -it / SLIDE_OFFSET_DIVISOR },
        modifier = modifier,
    ) {
        SuggestionsPanel(suggestions = suggestions, onSelect = onSelect)
    }
}

@Composable
private fun SuggestionsPanel(
    suggestions: List<SearchSuggestion>,
    onSelect: (insertText: String) -> Unit,
    modifier: Modifier = Modifier,
) {
    Surface(
        modifier = modifier.fillMaxWidth(),
        color = MaterialTheme.colorScheme.surface,
        shadowElevation = IndelibleSpacing.step4,
    ) {
        Column {
            suggestions.forEachIndexed { index, suggestion ->
                SuggestionRow(
                    suggestion = suggestion,
                    onClick = { onSelect(suggestion.insertText) },
                )
                if (index < suggestions.lastIndex) {
                    HorizontalDivider(
                        color = MaterialTheme.colorScheme.outlineVariant,
                        modifier = Modifier.padding(start = IndelibleSpacing.rowPaddingH),
                    )
                }
            }
        }
    }
}

@Composable
private fun SuggestionRow(
    suggestion: SearchSuggestion,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Row(
        modifier =
            modifier
                .fillMaxWidth()
                .clickable(onClick = onClick)
                .padding(
                    horizontal = IndelibleSpacing.rowPaddingH,
                    vertical = IndelibleSpacing.step12,
                ),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Icon(
            imageVector = suggestionKindIcon(suggestion.kind),
            contentDescription = null,
            modifier = Modifier.size(IndelibleSpacing.step16),
            tint = MaterialTheme.colorScheme.primary,
        )
        Spacer(modifier = Modifier.width(IndelibleSpacing.step12))
        Column(modifier = Modifier.weight(1f)) {
            Text(
                text = suggestion.label,
                style = MaterialTheme.typography.bodyLarge,
                color = MaterialTheme.colorScheme.onSurface,
            )
            if (suggestion.description != null) {
                Text(
                    text = suggestion.description,
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
        }
        Text(
            text = suggestion.insertText,
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f),
        )
    }
}

private fun suggestionKindIcon(kind: String): ImageVector =
    when (kind) {
        "tag" -> Icons.Filled.Label
        "collection" -> Icons.Filled.Folder
        "recent" -> Icons.Filled.History
        "sender" -> Icons.Filled.AlternateEmail
        "author" -> Icons.Filled.Person
        "list" -> Icons.Filled.Folder
        else -> Icons.Filled.TipsAndUpdates
    }

@Preview(showBackground = true, uiMode = 0x20)
@Composable
private fun SearchResultRowPreviewDark() {
    AppTheme(darkTheme = true) {
        Surface {
            SearchResultRow(
                result =
                    SearchResult(
                        documentId = "1",
                        title = "Designing Data-Intensive Applications",
                        contentType = "book",
                        resultKind = "item",
                        url = "https://dataintensive.net",
                        savedAt = Instant.parse("2026-01-01T00:00:00Z"),
                        updatedAt = Instant.parse("2026-01-01T00:00:00Z"),
                        score = 0.0,
                        snippet = "The definitive guide to <mark>distributed systems</mark>",
                    ),
                onClick = {},
            )
        }
    }
}
