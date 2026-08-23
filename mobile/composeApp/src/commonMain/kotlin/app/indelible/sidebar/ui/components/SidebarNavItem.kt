package app.indelible.sidebar.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
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
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import app.indelible.core.i18n.LocaleFormatters
import app.indelible.ui.theme.AppTheme
import app.indelible.ui.theme.IndelibleIcons
import app.indelible.ui.theme.IndelibleShape
import app.indelible.ui.theme.IndelibleSpacing
import app.indelible.ui.theme.geistMonoFontFamily

/**
 * A single navigation row in the drawer (prototype `.dw-item`).
 *
 * The leading slot holds either an [icon] (content types, smart lists) or a
 * coloured [dotColor] (collections) — they are mutually exclusive and share a
 * fixed-width box so every label left-aligns regardless of which is shown.
 * An [active] row fills with the accent-soft container and tints its label and
 * icon with the accent. [count] renders a trailing monospace tally when present.
 */
@Composable
fun SidebarNavItem(
    label: String,
    active: Boolean,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    icon: ImageVector? = null,
    iconTint: Color? = null,
    dotColor: Color? = null,
    count: Int? = null,
) {
    val background = if (active) MaterialTheme.colorScheme.primaryContainer else Color.Transparent
    val labelColor = if (active) MaterialTheme.colorScheme.onPrimaryContainer else MaterialTheme.colorScheme.onSurface
    val resolvedIconTint =
        when {
            active -> MaterialTheme.colorScheme.onPrimaryContainer
            iconTint != null -> iconTint
            else -> MaterialTheme.colorScheme.onSurfaceVariant
        }

    Row(
        modifier =
            modifier
                .fillMaxWidth()
                .clip(IndelibleShape.lg)
                .background(background)
                .clickable(onClick = onClick)
                .padding(horizontal = IndelibleSpacing.step12, vertical = IndelibleSpacing.step10),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Box(
            modifier = Modifier.size(IndelibleSpacing.step20),
            contentAlignment = Alignment.Center,
        ) {
            when {
                icon != null ->
                    Icon(
                        imageVector = icon,
                        contentDescription = null,
                        tint = resolvedIconTint,
                        modifier = Modifier.size(IndelibleSpacing.step20),
                    )
                dotColor != null ->
                    Box(
                        modifier =
                            Modifier
                                .size(IndelibleSpacing.step10)
                                .clip(IndelibleShape.xs)
                                .background(dotColor),
                    )
            }
        }
        Spacer(modifier = Modifier.width(IndelibleSpacing.step12))
        Text(
            text = label,
            style = MaterialTheme.typography.bodyLarge,
            fontWeight = if (active) FontWeight.SemiBold else FontWeight.Medium,
            color = labelColor,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
            modifier = Modifier.weight(1f),
        )
        if (count != null) {
            Spacer(modifier = Modifier.width(IndelibleSpacing.step8))
            Text(
                text = LocaleFormatters.number(count.toLong()),
                style = MaterialTheme.typography.labelSmall.copy(fontFamily = geistMonoFontFamily()),
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
    }
}

@Preview
@Composable
private fun SidebarNavItemPreviewLight() {
    AppTheme(darkTheme = false) {
        Surface {
            Column {
                SidebarNavItem(
                    label = "All items",
                    active = true,
                    onClick = {},
                    icon = IndelibleIcons.Grid,
                    count = 289,
                )
                SidebarNavItem(
                    label = "Articles",
                    active = false,
                    onClick = {},
                    icon = IndelibleIcons.Article,
                    count = 124,
                )
                SidebarNavItem(
                    label = "Reading list",
                    active = false,
                    onClick = {},
                    dotColor = MaterialTheme.colorScheme.primary,
                    count = 14,
                )
                SidebarNavItem(
                    label = "Unread",
                    active = false,
                    onClick = {},
                    icon = IndelibleIcons.SmartList,
                    iconTint = MaterialTheme.colorScheme.tertiary,
                    count = 47,
                )
            }
        }
    }
}

@Preview
@Composable
private fun SidebarNavItemPreviewDark() {
    AppTheme(darkTheme = true) {
        Surface {
            Column {
                SidebarNavItem(
                    label = "All items",
                    active = true,
                    onClick = {},
                    icon = IndelibleIcons.Grid,
                    count = 289,
                )
                SidebarNavItem(
                    label = "Videos",
                    active = false,
                    onClick = {},
                    icon = IndelibleIcons.Video,
                    count = 38,
                )
                SidebarNavItem(
                    label = "Work",
                    active = false,
                    onClick = {},
                    dotColor = MaterialTheme.colorScheme.primary,
                    count = 26,
                )
            }
        }
    }
}
