package app.indelible.home.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import app.indelible.home.model.HomeItem
import app.indelible.home.model.progressFraction
import app.indelible.ui.theme.AppTheme
import app.indelible.ui.theme.IndelibleShape
import app.indelible.ui.theme.IndelibleSpacing
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.home_minutes_left
import kotlin.math.roundToInt
import org.jetbrains.compose.resources.pluralStringResource

private val cardWidth = IndelibleSpacing.step96 + IndelibleSpacing.step56
private val coverHeight = IndelibleSpacing.step96 + IndelibleSpacing.step2

/**
 * Horizontally-scrolling rail of partially-read items ("Jump back in"). Each card
 * shows the cover with a progress sliver, the source, the title, and minutes left.
 * The rail bleeds to the screen edges via its own content padding.
 */
@Composable
fun JumpBackRail(
    items: List<HomeItem>,
    onItem: (HomeItem) -> Unit,
    modifier: Modifier = Modifier,
) {
    LazyRow(
        modifier = modifier.fillMaxWidth(),
        contentPadding = PaddingValues(horizontal = IndelibleSpacing.rowPaddingH),
        horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step12),
    ) {
        items(items, key = { it.id }) { item ->
            JumpBackCard(item = item, onClick = { onItem(item) })
        }
    }
}

@Composable
private fun JumpBackCard(
    item: HomeItem,
    onClick: () -> Unit,
) {
    val progress = item.progressFraction
    val minutesLeft = item.readingTimeMinutes?.let { (it * (1f - progress)).roundToInt() }
    Column(
        modifier =
            Modifier
                .width(cardWidth)
                .clip(IndelibleShape.lg)
                .clickable(onClick = onClick),
    ) {
        Box(
            modifier =
                Modifier
                    .fillMaxWidth()
                    .height(coverHeight),
        ) {
            HomeThumbnail(
                item = item,
                shape = IndelibleShape.lg,
                modifier = Modifier.matchParentSize(),
            )
            if (progress > 0f) {
                Box(
                    modifier =
                        Modifier
                            .align(Alignment.BottomStart)
                            .fillMaxWidth()
                            .height(IndelibleSpacing.step4)
                            .background(MaterialTheme.colorScheme.scrim.copy(alpha = 0.3f)),
                ) {
                    Box(
                        modifier =
                            Modifier
                                .fillMaxWidth(fraction = progress)
                                .height(IndelibleSpacing.step4)
                                .background(MaterialTheme.colorScheme.primary),
                    )
                }
            }
        }

        Spacer(Modifier.height(IndelibleSpacing.step8))

        if (!item.domain.isNullOrBlank()) {
            Text(
                text = item.domain.uppercase(),
                style = homeEyebrowStyle(),
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
            Spacer(Modifier.height(IndelibleSpacing.step2))
        }
        Text(
            text = item.title,
            style = MaterialTheme.typography.titleSmall,
            color = MaterialTheme.colorScheme.onSurface,
            maxLines = 2,
            overflow = TextOverflow.Ellipsis,
        )
        if (minutesLeft != null && minutesLeft > 0) {
            Spacer(Modifier.height(IndelibleSpacing.step2))
            Text(
                text = pluralStringResource(Res.plurals.home_minutes_left, minutesLeft, minutesLeft),
                style = MaterialTheme.typography.labelSmall,
                color = MaterialTheme.colorScheme.primary,
                maxLines = 1,
            )
        }
    }
}

private val previewJumpBack =
    listOf(
        previewHomeItem(
            id = "lib_a",
            title = "The Quiet Architecture of Attention",
            domain = "newyorker.com",
            progressPercent = 45f,
            readingTimeMinutes = 12,
        ),
        previewHomeItem(
            id = "lib_b",
            title = "Notes on Slow Software and the Craft of Patience",
            domain = "increment.com",
            progressPercent = 80f,
            readingTimeMinutes = 9,
        ),
    )

@Preview
@Composable
private fun JumpBackRailPreviewLight() {
    AppTheme(darkTheme = false) {
        Surface {
            JumpBackRail(items = previewJumpBack, onItem = {})
        }
    }
}

@Preview
@Composable
private fun JumpBackRailPreviewDark() {
    AppTheme(darkTheme = true) {
        Surface {
            JumpBackRail(items = previewJumpBack, onItem = {})
        }
    }
}
