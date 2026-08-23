package app.indelible.library.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Archive
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.SwipeToDismissBox
import androidx.compose.material3.SwipeToDismissBoxValue
import androidx.compose.material3.Text
import androidx.compose.material3.rememberSwipeToDismissBoxState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.tooling.preview.Preview
import app.indelible.core.model.LibraryItem
import app.indelible.ui.platform.rememberHapticTick
import app.indelible.ui.theme.AppTheme
import app.indelible.ui.theme.IndelibleSpacing
import app.indelible.ui.theme.IndelibleTheme
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.library_action_archive
import indelible.composeapp.generated.resources.library_action_delete
import kotlinx.datetime.Instant
import org.jetbrains.compose.resources.stringResource

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ItemSwipeableRow(
    item: LibraryItem,
    onTap: () -> Unit,
    onDelete: () -> Unit,
    onTriage: (String) -> Unit,
    modifier: Modifier = Modifier,
    showDivider: Boolean = true,
) {
    val dismissState = rememberSwipeToDismissBoxState()
    val hapticTick = rememberHapticTick()

    LaunchedEffect(dismissState.currentValue) {
        when (dismissState.currentValue) {
            SwipeToDismissBoxValue.StartToEnd -> {
                hapticTick()
                onTriage("archive")
            }
            SwipeToDismissBoxValue.EndToStart -> {
                hapticTick()
                onDelete()
            }
            SwipeToDismissBoxValue.Settled -> {}
        }
    }

    SwipeToDismissBox(
        state = dismissState,
        modifier = modifier,
        enableDismissFromStartToEnd = true,
        enableDismissFromEndToStart = true,
        backgroundContent = {
            val direction = dismissState.dismissDirection
            val (bgColor, icon, label, alignment) =
                when (direction) {
                    SwipeToDismissBoxValue.StartToEnd ->
                        Quad(
                            IndelibleTheme.colors.success,
                            Icons.Filled.Archive,
                            stringResource(Res.string.library_action_archive),
                            Alignment.CenterStart,
                        )
                    SwipeToDismissBoxValue.EndToStart ->
                        Quad(
                            MaterialTheme.colorScheme.error,
                            Icons.Filled.Delete,
                            stringResource(Res.string.library_action_delete),
                            Alignment.CenterEnd,
                        )
                    SwipeToDismissBoxValue.Settled -> return@SwipeToDismissBox
                }
            val contentColor =
                when (direction) {
                    SwipeToDismissBoxValue.StartToEnd -> IndelibleTheme.colors.onSuccess
                    else -> MaterialTheme.colorScheme.onError
                }
            Box(
                modifier = Modifier.fillMaxSize().background(bgColor),
                contentAlignment = alignment,
            ) {
                Column(
                    horizontalAlignment = Alignment.CenterHorizontally,
                    modifier = Modifier.padding(horizontal = IndelibleSpacing.step24),
                ) {
                    Icon(imageVector = icon, contentDescription = label, tint = contentColor)
                    Text(
                        text = label,
                        style = MaterialTheme.typography.bodySmall,
                        color = contentColor,
                    )
                }
            }
        },
    ) {
        LibraryItemRow(
            item = item,
            onClick = onTap,
            showDivider = showDivider,
        )
    }
}

private data class Quad<A, B, C, D>(
    val first: A,
    val second: B,
    val third: C,
    val fourth: D,
)

@Preview(showBackground = true)
@Composable
private fun ItemSwipeableRowPreviewLight() {
    AppTheme(darkTheme = false) {
        Surface {
            ItemSwipeableRow(
                item = previewSwipeItem(),
                onTap = {},
                onDelete = {},
                onTriage = {},
            )
        }
    }
}

@Preview(showBackground = true, uiMode = 0x20)
@Composable
private fun ItemSwipeableRowPreviewDark() {
    AppTheme(darkTheme = true) {
        Surface {
            ItemSwipeableRow(
                item = previewSwipeItem(),
                onTap = {},
                onDelete = {},
                onTriage = {},
            )
        }
    }
}

private fun previewSwipeItem() =
    LibraryItem(
        id = "lib_preview1",
        documentId = "doc_preview1",
        itemType = "article",
        triageState = "inbox",
        isFavorite = false,
        isShortlisted = false,
        title = "Understanding Coroutines in Kotlin",
        excerpt = "A comprehensive guide to async programming",
        domain = "kotlinlang.org",
        savedAt = Instant.parse("2024-01-15T12:00:00Z"),
        source = "url",
        createdAt = Instant.parse("2024-01-15T12:00:00Z"),
        updatedAt = Instant.parse("2024-01-15T12:00:00Z"),
    )
