package app.indelible.home.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.CheckCircle
import androidx.compose.material.icons.filled.LocalFireDepartment
import androidx.compose.material.icons.filled.Schedule
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
import app.indelible.home.viewmodel.StatIcon
import app.indelible.home.viewmodel.StatTile
import app.indelible.core.i18n.LocaleFormatters
import app.indelible.ui.theme.AppTheme
import app.indelible.ui.theme.IndelibleShape
import app.indelible.ui.theme.IndelibleSpacing
import app.indelible.ui.theme.IndelibleTheme
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.home_stat_finished
import indelible.composeapp.generated.resources.home_stat_read
import indelible.composeapp.generated.resources.home_stat_streak
import org.jetbrains.compose.resources.stringResource

/**
 * The three-up reading-stats grid below the hero: each tile pairs a tinted glyph
 * with a value and an uppercase label. Tiles share the row width equally.
 */
@Composable
fun StatsRow(
    stats: List<StatTile>,
    zeroed: Boolean = false,
    modifier: Modifier = Modifier,
) {
    Row(
        modifier = modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step10),
    ) {
        stats.forEach { tile -> StatTileCard(tile = tile, zeroed = zeroed) }
    }
}

@Composable
private fun RowScope.StatTileCard(
    tile: StatTile,
    zeroed: Boolean,
) {
    val tint = if (zeroed) MaterialTheme.colorScheme.onSurfaceVariant else statIconTint(tile.icon)
    Surface(
        shape = IndelibleShape.md,
        color = MaterialTheme.colorScheme.surfaceContainer,
        modifier = Modifier.weight(1f),
    ) {
        Column(modifier = Modifier.padding(IndelibleSpacing.step12)) {
            Box(
                modifier =
                    Modifier
                        .size(IndelibleSpacing.step28)
                        .clip(IndelibleShape.sm)
                        .background(tint.copy(alpha = 0.14f)),
                contentAlignment = Alignment.Center,
            ) {
                Icon(
                    imageVector = statIconVector(tile.icon),
                    contentDescription = null,
                    tint = tint,
                    modifier = Modifier.size(IndelibleSpacing.step16),
                )
            }
            Spacer(Modifier.height(IndelibleSpacing.step10))
            Text(
                text = LocaleFormatters.number(tile.value),
                style = MaterialTheme.typography.titleLarge.copy(fontWeight = FontWeight.Bold),
                color =
                    if (zeroed) {
                        MaterialTheme.colorScheme.onSurfaceVariant
                    } else {
                        MaterialTheme.colorScheme.onSurface
                    },
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
            Text(
                text = stringResource(tile.labelRes),
                style = homeEyebrowStyle(),
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
        }
    }
}

@Composable
private fun statIconTint(icon: StatIcon): Color =
    when (icon) {
        StatIcon.READING_TIME -> MaterialTheme.colorScheme.primary
        StatIcon.ITEMS_COMPLETED -> IndelibleTheme.colors.success
        StatIcon.STREAK -> IndelibleTheme.colors.warning
    }

private fun statIconVector(icon: StatIcon): ImageVector =
    when (icon) {
        StatIcon.READING_TIME -> Icons.Filled.Schedule
        StatIcon.ITEMS_COMPLETED -> Icons.Filled.CheckCircle
        StatIcon.STREAK -> Icons.Filled.LocalFireDepartment
    }

private val previewStats =
    listOf(
        StatTile(labelRes = Res.string.home_stat_read, value = 4, icon = StatIcon.READING_TIME),
        StatTile(labelRes = Res.string.home_stat_finished, value = 9, icon = StatIcon.ITEMS_COMPLETED),
        StatTile(labelRes = Res.string.home_stat_streak, value = 12, icon = StatIcon.STREAK),
    )

@Preview
@Composable
private fun StatsRowPreviewLight() {
    AppTheme(darkTheme = false) {
        Surface {
            StatsRow(stats = previewStats, modifier = Modifier.padding(IndelibleSpacing.step16))
        }
    }
}

@Preview
@Composable
private fun StatsRowPreviewDark() {
    AppTheme(darkTheme = true) {
        Surface {
            StatsRow(stats = previewStats, modifier = Modifier.padding(IndelibleSpacing.step16))
        }
    }
}
