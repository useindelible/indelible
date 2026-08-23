package app.indelible.home.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
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
import app.indelible.ui.components.IndelibleButton
import app.indelible.ui.theme.AppTheme
import app.indelible.ui.theme.IndelibleShape
import app.indelible.ui.theme.IndelibleSpacing
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.home_continue_reading
import indelible.composeapp.generated.resources.home_minutes_left
import indelible.composeapp.generated.resources.home_pick_up
import indelible.composeapp.generated.resources.home_progress_percent
import indelible.composeapp.generated.resources.home_resume
import org.jetbrains.compose.resources.pluralStringResource
import org.jetbrains.compose.resources.stringResource
import kotlin.math.roundToInt

/**
 * The "Continue reading" hero card: the single most recently-in-progress item,
 * surfaced large at the top of the dashboard with a resume affordance.
 */
private const val PERCENT_SCALE = 100

@Composable
fun ContinueReadingHero(
    item: HomeItem,
    onResume: () -> Unit,
    onOpen: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val progress = item.progressFraction
    val percent = (progress * PERCENT_SCALE).roundToInt()
    val minutesLeft = item.readingTimeMinutes?.let { (it * (1f - progress)).roundToInt() }
    val footLabel =
        if (minutesLeft != null && minutesLeft > 0) {
            pluralStringResource(Res.plurals.home_minutes_left, minutesLeft, minutesLeft)
        } else {
            stringResource(Res.string.home_pick_up)
        }

    Surface(
        onClick = onOpen,
        shape = IndelibleShape.xxl,
        color = MaterialTheme.colorScheme.surfaceContainerHigh,
        modifier = modifier.fillMaxWidth(),
    ) {
        Column(modifier = Modifier.padding(IndelibleSpacing.step20)) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Text(
                    text = stringResource(Res.string.home_continue_reading),
                    style = homeEyebrowStyle(),
                    color = MaterialTheme.colorScheme.primary,
                )
                Text(
                    text = stringResource(Res.string.home_progress_percent, percent),
                    style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }

            Spacer(Modifier.height(IndelibleSpacing.step14))

            Row(verticalAlignment = Alignment.Top) {
                HomeThumbnail(
                    item = item,
                    shape = IndelibleShape.lg,
                    modifier = Modifier.size(IndelibleSpacing.step56),
                )
                Spacer(Modifier.width(IndelibleSpacing.step14))
                Column(modifier = Modifier.weight(1f)) {
                    if (!item.domain.isNullOrBlank()) {
                        Text(
                            text = item.domain,
                            style = MaterialTheme.typography.labelSmall,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                            maxLines = 1,
                            overflow = TextOverflow.Ellipsis,
                        )
                        Spacer(Modifier.height(IndelibleSpacing.step2))
                    }
                    Text(
                        text = item.title,
                        style = MaterialTheme.typography.titleLarge,
                        color = MaterialTheme.colorScheme.onSurface,
                        maxLines = 2,
                        overflow = TextOverflow.Ellipsis,
                    )
                }
            }

            Spacer(Modifier.height(IndelibleSpacing.step14))

            HeroProgressBar(progress = progress)

            Spacer(Modifier.height(IndelibleSpacing.step14))

            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Text(
                    text = footLabel,
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
                IndelibleButton(
                    text = stringResource(Res.string.home_resume),
                    onClick = onResume,
                    compact = true,
                )
            }
        }
    }
}

@Composable
private fun HeroProgressBar(
    progress: Float,
    modifier: Modifier = Modifier,
) {
    Box(
        modifier =
            modifier
                .fillMaxWidth()
                .height(IndelibleSpacing.step6)
                .clip(IndelibleShape.full)
                .background(MaterialTheme.colorScheme.outlineVariant),
    ) {
        Box(
            modifier =
                Modifier
                    .fillMaxWidth(fraction = progress)
                    .height(IndelibleSpacing.step6)
                    .clip(IndelibleShape.full)
                    .background(MaterialTheme.colorScheme.primary),
        )
    }
}

@Preview
@Composable
private fun ContinueReadingHeroPreviewLight() {
    AppTheme(darkTheme = false) {
        Surface {
            ContinueReadingHero(
                item = previewHomeItem(progressPercent = 62f),
                onResume = {},
                onOpen = {},
                modifier = Modifier.padding(IndelibleSpacing.step16),
            )
        }
    }
}

@Preview
@Composable
private fun ContinueReadingHeroPreviewDark() {
    AppTheme(darkTheme = true) {
        Surface {
            ContinueReadingHero(
                item = previewHomeItem(progressPercent = 34f),
                onResume = {},
                onOpen = {},
                modifier = Modifier.padding(IndelibleSpacing.step16),
            )
        }
    }
}
