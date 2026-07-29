package app.indelible.reader.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.border
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
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.SegmentedButton
import androidx.compose.material3.SegmentedButtonDefaults
import androidx.compose.material3.SingleChoiceSegmentedButtonRow
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.tooling.preview.Preview
import app.indelible.reader.model.HighlightColor
import app.indelible.reader.model.HighlightStyle
import app.indelible.ui.theme.AppTheme
import app.indelible.ui.theme.IndelibleSpacing

/**
 * Highlight panel: picks the default color and paint style for new highlights.
 * Color selection updates the reader's default; style flips the WebView's
 * highlight rendering between a soft fill and a bare underline edge.
 */
@Composable
fun HighlightStylePanel(
    selectedColor: HighlightColor,
    style: HighlightStyle,
    onColorSelected: (HighlightColor) -> Unit,
    onStyleSelected: (HighlightStyle) -> Unit,
    modifier: Modifier = Modifier,
) {
    Column(modifier = modifier.fillMaxWidth()) {
        SectionLabel("Color")
        Spacer(modifier = Modifier.height(IndelibleSpacing.step8))
        Row(horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step12)) {
            HighlightColor.entries.forEach { color ->
                ColorChoice(
                    color = highlightColorToCompose(color),
                    selected = selectedColor == color,
                    contentDescription = color.apiValue,
                    onClick = { onColorSelected(color) },
                )
            }
        }

        Spacer(modifier = Modifier.height(IndelibleSpacing.sectionGap))

        SectionLabel("Style")
        Spacer(modifier = Modifier.height(IndelibleSpacing.step8))
        val styles = HighlightStyle.entries.toTypedArray()
        val styleLabels = arrayOf("Fill", "Underline")
        SingleChoiceSegmentedButtonRow(modifier = Modifier.fillMaxWidth()) {
            styles.forEachIndexed { index, candidate ->
                SegmentedButton(
                    selected = style == candidate,
                    onClick = { onStyleSelected(candidate) },
                    shape = SegmentedButtonDefaults.itemShape(index, styles.size),
                ) {
                    Text(text = styleLabels[index], style = MaterialTheme.typography.bodySmall)
                }
            }
        }

        Spacer(modifier = Modifier.height(IndelibleSpacing.step12))

        Text(
            text = "Sets the default color for new highlights.",
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
    }
}

@Composable
private fun SectionLabel(text: String) {
    Text(
        text = text,
        style = MaterialTheme.typography.labelMedium,
        color = MaterialTheme.colorScheme.onSurfaceVariant,
    )
}

@Composable
private fun ColorChoice(
    color: Color,
    selected: Boolean,
    contentDescription: String,
    onClick: () -> Unit,
) {
    val ring =
        if (selected) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.outlineVariant
    Box(
        modifier =
            Modifier
                .size(IndelibleSpacing.step40)
                .clip(CircleShape)
                .background(color)
                .border(IndelibleSpacing.step2, ring, CircleShape)
                .clickable(onClickLabel = contentDescription, onClick = onClick),
    )
}

@Preview
@Composable
private fun HighlightStylePanelPreviewLight() {
    AppTheme(darkTheme = false) {
        Surface {
            HighlightStylePanel(
                selectedColor = HighlightColor.YELLOW,
                style = HighlightStyle.FILL,
                onColorSelected = {},
                onStyleSelected = {},
                modifier = Modifier.padding(IndelibleSpacing.screenPaddingH),
            )
        }
    }
}

@Preview
@Composable
private fun HighlightStylePanelPreviewDark() {
    AppTheme(darkTheme = true) {
        Surface {
            HighlightStylePanel(
                selectedColor = HighlightColor.GREEN,
                style = HighlightStyle.UNDERLINE,
                onColorSelected = {},
                onStyleSelected = {},
                modifier = Modifier.padding(IndelibleSpacing.screenPaddingH),
            )
        }
    }
}
