package app.indelible.reader.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import app.indelible.reader.model.HighlightColor
import app.indelible.ui.platform.rememberHapticTick
import app.indelible.ui.theme.AppTheme
import app.indelible.ui.theme.HighlightBlueBorder
import app.indelible.ui.theme.HighlightGreenBorder
import app.indelible.ui.theme.HighlightPinkBorder
import app.indelible.ui.theme.HighlightPurpleBorder
import app.indelible.ui.theme.HighlightYellowBorder
import app.indelible.ui.theme.IndelibleIcons
import app.indelible.ui.theme.IndelibleSpacing
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.reader_action_copy
import indelible.composeapp.generated.resources.reader_action_note
import indelible.composeapp.generated.resources.reader_action_tag
import indelible.composeapp.generated.resources.reader_color_blue
import indelible.composeapp.generated.resources.reader_color_green
import indelible.composeapp.generated.resources.reader_color_pink
import indelible.composeapp.generated.resources.reader_color_purple
import indelible.composeapp.generated.resources.reader_color_yellow
import org.jetbrains.compose.resources.StringResource
import org.jetbrains.compose.resources.stringResource

@Composable
fun HighlightToolbar(
    onColorSelected: (HighlightColor) -> Unit,
    onTagSelected: () -> Unit,
    onNoteSelected: () -> Unit,
    onCopySelected: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val hapticTick = rememberHapticTick()
    Surface(
        modifier = modifier,
        shape = MaterialTheme.shapes.medium,
        shadowElevation = IndelibleSpacing.step4,
        color = MaterialTheme.colorScheme.surfaceContainer,
    ) {
        Row(
            modifier =
                Modifier.padding(
                    horizontal = IndelibleSpacing.step10,
                    vertical = IndelibleSpacing.step8,
                ),
            horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step2),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            HighlightColor.entries.forEach { color ->
                ColorDot(
                    color = highlightColorToCompose(color),
                    contentDescription = stringResource(highlightColorLabelRes(color)),
                    onClick = {
                        hapticTick()
                        onColorSelected(color)
                    },
                )
            }

            Box(
                modifier =
                    Modifier
                        .padding(horizontal = IndelibleSpacing.step6)
                        .width(1.dp)
                        .height(IndelibleSpacing.step16)
                        .background(MaterialTheme.colorScheme.outlineVariant),
            )

            ToolbarTextButton(label = stringResource(Res.string.reader_action_copy), onClick = onCopySelected)

            Row(
                modifier =
                    Modifier
                        .clip(MaterialTheme.shapes.small)
                        .clickable(onClick = onTagSelected)
                        .padding(horizontal = IndelibleSpacing.step8, vertical = IndelibleSpacing.step4),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step4),
            ) {
                Icon(
                    imageVector = IndelibleIcons.Tag,
                    contentDescription = null,
                    tint = MaterialTheme.colorScheme.onSurfaceVariant,
                    modifier = Modifier.size(12.dp),
                )
                Text(
                    text = stringResource(Res.string.reader_action_tag),
                    style = MaterialTheme.typography.labelMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }

            ToolbarTextButton(label = stringResource(Res.string.reader_action_note), onClick = onNoteSelected)
        }
    }
}

@Composable
private fun ToolbarTextButton(
    label: String,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Text(
        text = label,
        style = MaterialTheme.typography.labelMedium,
        color = MaterialTheme.colorScheme.onSurfaceVariant,
        modifier =
            modifier
                .clip(MaterialTheme.shapes.small)
                .clickable(onClick = onClick)
                .padding(horizontal = IndelibleSpacing.step8, vertical = IndelibleSpacing.step4),
    )
}

@Composable
private fun ColorDot(
    color: Color,
    contentDescription: String,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val description = contentDescription
    Box(
        modifier =
            modifier
                .size(IndelibleSpacing.step32)
                .clip(CircleShape)
                .clickable(onClick = onClick)
                .semantics { this.contentDescription = description },
        contentAlignment = Alignment.Center,
    ) {
        Box(
            modifier =
                Modifier
                    .size(IndelibleSpacing.step16)
                    .clip(CircleShape)
                    .background(color),
        )
    }
}

fun highlightColorToCompose(color: HighlightColor): Color =
    when (color) {
        HighlightColor.YELLOW -> HighlightYellowBorder
        HighlightColor.BLUE -> HighlightBlueBorder
        HighlightColor.GREEN -> HighlightGreenBorder
        HighlightColor.PINK -> HighlightPinkBorder
        HighlightColor.PURPLE -> HighlightPurpleBorder
    }

internal fun highlightColorLabelRes(color: HighlightColor): StringResource =
    when (color) {
        HighlightColor.YELLOW -> Res.string.reader_color_yellow
        HighlightColor.BLUE -> Res.string.reader_color_blue
        HighlightColor.GREEN -> Res.string.reader_color_green
        HighlightColor.PINK -> Res.string.reader_color_pink
        HighlightColor.PURPLE -> Res.string.reader_color_purple
    }

fun highlightColorNameToCompose(colorName: String): Color =
    when (colorName) {
        "Yellow" -> HighlightYellowBorder
        "Blue" -> HighlightBlueBorder
        "Green" -> HighlightGreenBorder
        "Pink" -> HighlightPinkBorder
        "Purple" -> HighlightPurpleBorder
        else -> HighlightYellowBorder
    }

@Preview(showBackground = true)
@Composable
private fun HighlightToolbarPreviewLight() {
    AppTheme(darkTheme = false) {
        HighlightToolbar(
            onColorSelected = {},
            onTagSelected = {},
            onNoteSelected = {},
            onCopySelected = {},
            modifier = Modifier.padding(IndelibleSpacing.step16),
        )
    }
}

@Preview(showBackground = true)
@Composable
private fun HighlightToolbarPreviewDark() {
    AppTheme(darkTheme = true) {
        HighlightToolbar(
            onColorSelected = {},
            onTagSelected = {},
            onNoteSelected = {},
            onCopySelected = {},
            modifier = Modifier.padding(IndelibleSpacing.step16),
        )
    }
}
