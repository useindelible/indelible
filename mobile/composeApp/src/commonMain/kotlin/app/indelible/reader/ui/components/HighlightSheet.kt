package app.indelible.reader.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ContentCopy
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material.icons.filled.Edit
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.tooling.preview.Preview
import app.indelible.reader.model.HighlightColor
import app.indelible.reader.model.HighlightData
import app.indelible.reader.model.HighlightLocator
import app.indelible.ui.theme.AppTheme
import app.indelible.ui.theme.IndelibleIcons
import app.indelible.ui.theme.IndelibleSpacing
import kotlinx.datetime.Instant

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun HighlightSheet(
    highlight: HighlightData,
    onColorChanged: (HighlightColor) -> Unit,
    onEditNote: () -> Unit,
    onTagsSelected: () -> Unit,
    onDelete: () -> Unit,
    onCopy: () -> Unit,
    onDismiss: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true)

    ModalBottomSheet(
        onDismissRequest = onDismiss,
        sheetState = sheetState,
        modifier = modifier,
    ) {
        HighlightSheetContent(
            highlight = highlight,
            onColorChanged = onColorChanged,
            onEditNote = onEditNote,
            onTagsSelected = onTagsSelected,
            onDelete = onDelete,
            onCopy = onCopy,
        )
    }
}

@Composable
private fun HighlightSheetContent(
    highlight: HighlightData,
    onColorChanged: (HighlightColor) -> Unit,
    onEditNote: () -> Unit,
    onTagsSelected: () -> Unit,
    onDelete: () -> Unit,
    onCopy: () -> Unit,
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
            text = highlight.textContent,
            style = MaterialTheme.typography.bodyLarge,
            color = MaterialTheme.colorScheme.onSurface,
            maxLines = 3,
        )

        Spacer(modifier = Modifier.height(IndelibleSpacing.step16))
        HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant)
        Spacer(modifier = Modifier.height(IndelibleSpacing.step16))

        Text(
            text = "Color",
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
        Spacer(modifier = Modifier.height(IndelibleSpacing.step8))

        Row(
            horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step8),
        ) {
            HighlightColor.entries.forEach { color ->
                val isSelected = color.apiValue == highlight.color
                Box(
                    modifier =
                        Modifier
                            .size(IndelibleSpacing.touchTarget)
                            .clip(CircleShape)
                            .clickable { onColorChanged(color) },
                    contentAlignment = Alignment.Center,
                ) {
                    Box(
                        modifier =
                            Modifier
                                .size(
                                    if (isSelected) IndelibleSpacing.step32 else IndelibleSpacing.step24,
                                ).clip(CircleShape)
                                .background(highlightColorToCompose(color)),
                    )
                }
            }
        }

        Spacer(modifier = Modifier.height(IndelibleSpacing.step16))
        HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant)

        SheetAction(
            icon = { Icon(Icons.Filled.Edit, contentDescription = null) },
            label = if (highlight.note != null) "Edit Note" else "Add Note",
            onClick = onEditNote,
        )

        val tagCount = highlight.tags.size
        SheetAction(
            icon = { Icon(IndelibleIcons.Tag, contentDescription = null) },
            label = if (tagCount > 0) "Tags ($tagCount)" else "Add Tags",
            onClick = onTagsSelected,
        )

        SheetAction(
            icon = { Icon(Icons.Filled.ContentCopy, contentDescription = null) },
            label = "Copy",
            onClick = onCopy,
        )

        SheetAction(
            icon = {
                Icon(
                    Icons.Filled.Delete,
                    contentDescription = null,
                    tint = MaterialTheme.colorScheme.error,
                )
            },
            label = "Delete",
            labelColor = MaterialTheme.colorScheme.error,
            onClick = onDelete,
        )

        Spacer(modifier = Modifier.height(IndelibleSpacing.step16))
    }
}

@Composable
private fun SheetAction(
    icon: @Composable () -> Unit,
    label: String,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    labelColor: androidx.compose.ui.graphics.Color = MaterialTheme.colorScheme.onSurface,
) {
    Row(
        modifier =
            modifier
                .fillMaxWidth()
                .clickable(onClick = onClick)
                .padding(vertical = IndelibleSpacing.rowPaddingV),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step16),
    ) {
        icon()
        Text(
            text = label,
            style = MaterialTheme.typography.bodyLarge,
            color = labelColor,
        )
    }
}

@Preview(showBackground = true)
@Composable
private fun HighlightSheetContentPreviewLight() {
    AppTheme(darkTheme = false) {
        Surface {
            HighlightSheetContent(
                highlight = previewHighlight(),
                onColorChanged = {},
                onEditNote = {},
                onTagsSelected = {},
                onDelete = {},
                onCopy = {},
            )
        }
    }
}

@Preview(showBackground = true)
@Composable
private fun HighlightSheetContentPreviewDark() {
    AppTheme(darkTheme = true) {
        Surface {
            HighlightSheetContent(
                highlight = previewHighlight(color = "Blue"),
                onColorChanged = {},
                onEditNote = {},
                onTagsSelected = {},
                onDelete = {},
                onCopy = {},
            )
        }
    }
}

private fun previewHighlight(color: String = "Yellow") =
    HighlightData(
        id = "hlt_preview",
        documentId = "doc_preview",
        color = color,
        textContent = "The rapid advancement of open-source AI models has fundamentally changed" +
            " the competitive landscape.",
        locator = HighlightLocator(type = "html", startOffset = 0, endOffset = 95),
        tags = emptyList(),
        createdAt = Instant.parse("2024-01-15T12:00:00Z"),
        updatedAt = Instant.parse("2024-01-15T12:00:00Z"),
    )
