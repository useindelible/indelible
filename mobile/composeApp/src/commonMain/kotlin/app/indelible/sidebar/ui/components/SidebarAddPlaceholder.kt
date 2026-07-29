package app.indelible.sidebar.ui.components

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.geometry.CornerRadius
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.PathEffect
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import app.indelible.ui.theme.AppTheme
import app.indelible.ui.theme.IndelibleIcons
import app.indelible.ui.theme.IndelibleShape
import app.indelible.ui.theme.IndelibleSpacing

/**
 * Ghost "New collection" / "New smart list" row shown when a group is empty
 * (prototype `.dw-add`): a dashed hairline outline with a leading plus glyph.
 */
@Composable
fun SidebarAddPlaceholder(
    label: String,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val borderColor = MaterialTheme.colorScheme.outline
    Row(
        modifier =
            modifier
                .fillMaxWidth()
                .drawBehind {
                    // 1px hairline outline — a draw primitive, not a layout token. Inset by
                    // half the stroke so the dashed edge stays fully inside the clipped bounds.
                    val sw = 1.dp.toPx()
                    drawRoundRect(
                        color = borderColor,
                        topLeft = Offset(sw / 2f, sw / 2f),
                        size = Size(size.width - sw, size.height - sw),
                        cornerRadius = CornerRadius(IndelibleSpacing.step12.toPx()),
                        style =
                            Stroke(
                                width = sw,
                                pathEffect =
                                    PathEffect.dashPathEffect(
                                        floatArrayOf(
                                            IndelibleSpacing.step6.toPx(),
                                            IndelibleSpacing.step4.toPx(),
                                        ),
                                    ),
                            ),
                    )
                }
                .clip(IndelibleShape.lg)
                .clickable(onClick = onClick)
                .padding(horizontal = IndelibleSpacing.step12, vertical = IndelibleSpacing.step10),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Icon(
            imageVector = IndelibleIcons.Plus,
            contentDescription = null,
            tint = MaterialTheme.colorScheme.onSurfaceVariant,
            modifier = Modifier.size(IndelibleSpacing.step20),
        )
        Spacer(modifier = Modifier.width(IndelibleSpacing.step12))
        Text(
            text = label,
            style = MaterialTheme.typography.bodyLarge,
            fontWeight = FontWeight.Medium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
    }
}

@Preview
@Composable
private fun SidebarAddPlaceholderPreviewLight() {
    AppTheme(darkTheme = false) {
        Surface {
            Column {
                SidebarAddPlaceholder(label = "New collection", onClick = {})
                SidebarAddPlaceholder(label = "New smart list", onClick = {})
            }
        }
    }
}

@Preview
@Composable
private fun SidebarAddPlaceholderPreviewDark() {
    AppTheme(darkTheme = true) {
        Surface {
            SidebarAddPlaceholder(label = "New collection", onClick = {})
        }
    }
}
