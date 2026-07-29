package app.indelible.sidebar.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.Dp
import app.indelible.ui.theme.AppTheme
import app.indelible.ui.theme.IndelibleIcons
import app.indelible.ui.theme.IndelibleShape
import app.indelible.ui.theme.IndelibleSpacing

/**
 * Drawer footer (prototype `.dw-foot`): a hairline divider over two equal-width
 * Settings / Trash tiles. The hosting `ModalDrawerSheet` already reserves the bottom
 * safe-area inset, so the row adds only a symmetric design pad and does NOT re-apply a
 * navigation-bar inset — doing so stacked a second gap below the tiles.
 */
@Composable
fun SidebarFooter(
    onSettings: () -> Unit,
    onTrash: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Column(modifier = modifier.fillMaxWidth()) {
        HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant)
        Row(
            modifier =
                Modifier
                    .fillMaxWidth()
                    .padding(
                        start = IndelibleSpacing.step16,
                        end = IndelibleSpacing.step16,
                        top = IndelibleSpacing.step14,
                        bottom = IndelibleSpacing.step14,
                    ),
            horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step8),
        ) {
            FooterTile(icon = IndelibleIcons.Settings, label = "Settings", onClick = onSettings)
            FooterTile(icon = IndelibleIcons.Trash, label = "Trash", onClick = onTrash)
        }
    }
}

@Composable
private fun RowScope.FooterTile(
    icon: ImageVector,
    label: String,
    onClick: () -> Unit,
) {
    Row(
        modifier =
            Modifier
                .weight(1f)
                .clip(IndelibleShape.lg)
                .background(MaterialTheme.colorScheme.surfaceVariant)
                .border(Dp.Hairline, MaterialTheme.colorScheme.outline, IndelibleShape.lg)
                .clickable(onClick = onClick)
                .padding(vertical = IndelibleSpacing.step12),
        horizontalArrangement = Arrangement.Center,
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Icon(
            imageVector = icon,
            contentDescription = null,
            tint = MaterialTheme.colorScheme.onSurfaceVariant,
            modifier = Modifier.size(IndelibleSpacing.step16),
        )
        Spacer(modifier = Modifier.width(IndelibleSpacing.step8))
        Text(
            text = label,
            style = MaterialTheme.typography.bodyMedium,
            fontWeight = FontWeight.Medium,
            color = MaterialTheme.colorScheme.onSurface,
        )
    }
}

@Preview
@Composable
private fun SidebarFooterPreviewLight() {
    AppTheme(darkTheme = false) {
        Surface {
            SidebarFooter(onSettings = {}, onTrash = {})
        }
    }
}

@Preview
@Composable
private fun SidebarFooterPreviewDark() {
    AppTheme(darkTheme = true) {
        Surface {
            SidebarFooter(onSettings = {}, onTrash = {})
        }
    }
}
