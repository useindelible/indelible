package app.indelible.search.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.History
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.SwipeToDismissBox
import androidx.compose.material3.SwipeToDismissBoxValue
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.rememberSwipeToDismissBoxState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.tooling.preview.Preview
import app.indelible.search.model.RecentSearch
import app.indelible.search.viewmodel.SearchState
import app.indelible.ui.theme.AppTheme
import app.indelible.ui.theme.IndelibleShape
import app.indelible.ui.theme.IndelibleSpacing
import kotlinx.datetime.Instant

private val SYNTAX_HINTS =
    listOf(
        "tag:",
        "type:",
        "collection:",
        "author:",
        "before:",
        "after:",
        "is:",
        "has:",
        "url:",
        "!tag:",
    )

private val Number.em get() =
    androidx.compose.ui.unit
        .TextUnit(this.toFloat(), androidx.compose.ui.unit.TextUnitType.Em)

@OptIn(ExperimentalLayoutApi::class, ExperimentalMaterial3Api::class)
@Composable
internal fun IdleContent(
    state: SearchState,
    onRecentClick: (String) -> Unit,
    onRecentDelete: (String) -> Unit,
    onClearAll: () -> Unit,
    onSyntaxPillClick: (String) -> Unit,
    modifier: Modifier = Modifier,
) {
    LazyColumn(modifier = modifier.fillMaxSize()) {
        if (state.recentSearches.isNotEmpty()) {
            item {
                Row(
                    modifier =
                        Modifier
                            .fillMaxWidth()
                            .padding(
                                start = IndelibleSpacing.rowPaddingH,
                                end = IndelibleSpacing.step8,
                                top = IndelibleSpacing.step20,
                                bottom = IndelibleSpacing.step8,
                            ),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    Text(
                        text = "RECENT SEARCHES",
                        style =
                            MaterialTheme.typography.labelSmall.copy(
                                letterSpacing = 0.06.em,
                            ),
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                    TextButton(onClick = onClearAll) {
                        Text(
                            text = "Clear All",
                            style = MaterialTheme.typography.bodySmall,
                            color = MaterialTheme.colorScheme.primary,
                        )
                    }
                }
            }
            items(items = state.recentSearches, key = { it.id }) { recent ->
                RecentSearchItem(
                    recent = recent,
                    onTap = { onRecentClick(recent.query) },
                    onDelete = { onRecentDelete(recent.id) },
                )
            }
        }

        item {
            Text(
                text = "FILTER SYNTAX",
                style =
                    MaterialTheme.typography.labelSmall.copy(
                        letterSpacing = 0.06.em,
                    ),
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                modifier =
                    Modifier.padding(
                        start = IndelibleSpacing.rowPaddingH,
                        end = IndelibleSpacing.rowPaddingH,
                        top = IndelibleSpacing.step20,
                        bottom = IndelibleSpacing.step8,
                    ),
            )
            FlowRow(
                modifier =
                    Modifier.padding(
                        horizontal = IndelibleSpacing.rowPaddingH,
                        vertical = IndelibleSpacing.step4,
                    ),
                horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step8),
                verticalArrangement = Arrangement.spacedBy(IndelibleSpacing.step8),
            ) {
                SYNTAX_HINTS.forEach { hint ->
                    SyntaxPill(text = hint, onClick = { onSyntaxPillClick(hint) })
                }
            }
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun RecentSearchItem(
    recent: RecentSearch,
    onTap: () -> Unit,
    onDelete: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val dismissState = rememberSwipeToDismissBoxState()

    LaunchedEffect(dismissState.currentValue) {
        if (dismissState.currentValue == SwipeToDismissBoxValue.EndToStart) {
            onDelete()
        }
    }

    SwipeToDismissBox(
        state = dismissState,
        modifier = modifier,
        enableDismissFromStartToEnd = false,
        enableDismissFromEndToStart = true,
        backgroundContent = {
            Box(
                modifier =
                    Modifier
                        .fillMaxSize()
                        .background(MaterialTheme.colorScheme.error)
                        .padding(end = IndelibleSpacing.rowPaddingH),
                contentAlignment = Alignment.CenterEnd,
            ) {
                Text(
                    text = "Delete",
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onError,
                )
            }
        },
    ) {
        Row(
            modifier =
                Modifier
                    .fillMaxWidth()
                    .clickable(onClick = onTap)
                    .background(MaterialTheme.colorScheme.surface)
                    .padding(
                        horizontal = IndelibleSpacing.rowPaddingH,
                        vertical = IndelibleSpacing.step12,
                    ),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Icon(
                imageVector = Icons.Filled.History,
                contentDescription = null,
                modifier = Modifier.size(IndelibleSpacing.step16),
                tint = MaterialTheme.colorScheme.onSurfaceVariant,
            )
            Spacer(modifier = Modifier.width(IndelibleSpacing.step12))
            Text(
                text = recent.query,
                style = MaterialTheme.typography.bodyLarge,
                color = MaterialTheme.colorScheme.onSurface,
                modifier = Modifier.weight(1f),
                maxLines = 1,
            )
        }
        HorizontalDivider(
            color = MaterialTheme.colorScheme.outlineVariant,
            modifier = Modifier.padding(start = IndelibleSpacing.rowPaddingH),
        )
    }
}

@Composable
private fun SyntaxPill(
    text: String,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Box(
        modifier =
            modifier
                .clip(IndelibleShape.full)
                .background(MaterialTheme.colorScheme.surfaceVariant)
                .clickable(onClick = onClick)
                .padding(horizontal = IndelibleSpacing.step12, vertical = IndelibleSpacing.step6),
    ) {
        Text(
            text = text,
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
    }
}

@Preview(showBackground = true)
@Composable
private fun SearchScreenIdlePreviewLight() {
    AppTheme(darkTheme = false) {
        Surface {
            IdleContent(
                state =
                    SearchState(
                        recentSearches =
                            listOf(
                                RecentSearch(
                                    id = "1",
                                    query = "distributed systems",
                                    normalizedQuery = "distributed systems",
                                    lastSearchedAt = Instant.parse("2026-04-01T10:00:00Z"),
                                    createdAt = Instant.parse("2026-04-01T10:00:00Z"),
                                    updatedAt = Instant.parse("2026-04-01T10:00:00Z"),
                                ),
                                RecentSearch(
                                    id = "2",
                                    query = "tag:research type:pdf",
                                    normalizedQuery = "tag:research type:pdf",
                                    lastSearchedAt = Instant.parse("2026-04-01T09:00:00Z"),
                                    createdAt = Instant.parse("2026-04-01T09:00:00Z"),
                                    updatedAt = Instant.parse("2026-04-01T09:00:00Z"),
                                ),
                            ),
                    ),
                onRecentClick = {},
                onRecentDelete = {},
                onClearAll = {},
                onSyntaxPillClick = {},
            )
        }
    }
}
