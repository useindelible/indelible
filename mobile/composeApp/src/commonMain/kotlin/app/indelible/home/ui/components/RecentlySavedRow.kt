package app.indelible.home.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.material3.HorizontalDivider
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
import indelible.composeapp.generated.resources.home_reading_minutes
import kotlinx.datetime.Instant
import org.jetbrains.compose.resources.pluralStringResource

/**
 * A "Recently saved" list row: large cover, an unread dot when the item has never
 * been opened, an uppercase source/length eyebrow, title, excerpt, and a progress
 * sliver when partially read. Structurally mirrors the library list row.
 */
@Composable
fun RecentlySavedRow(
    item: HomeItem,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    showDivider: Boolean = true,
) {
    Column(modifier = modifier.background(MaterialTheme.colorScheme.surface)) {
        Row(
            modifier =
                Modifier
                    .fillMaxWidth()
                    .clickable(onClick = onClick)
                    .padding(
                        horizontal = IndelibleSpacing.rowPaddingH,
                        vertical = IndelibleSpacing.rowPaddingV,
                    ),
            verticalAlignment = Alignment.Top,
        ) {
            Box(modifier = Modifier.size(IndelibleSpacing.step96)) {
                HomeThumbnail(
                    item = item,
                    shape = IndelibleShape.xl,
                    modifier = Modifier.matchParentSize(),
                )
                if (item.lastReadAt == null) {
                    Box(
                        modifier =
                            Modifier
                                .size(IndelibleSpacing.step14)
                                .offset(x = -IndelibleSpacing.step4, y = -IndelibleSpacing.step4)
                                .align(Alignment.TopStart)
                                .background(
                                    color = MaterialTheme.colorScheme.primary,
                                    shape = IndelibleShape.full,
                                ),
                    )
                }
            }
            Spacer(Modifier.width(IndelibleSpacing.rowContentGap))
            Column(modifier = Modifier.weight(1f)) {
                val eyebrow = recentlySavedEyebrow(item)
                if (eyebrow.isNotBlank()) {
                    Text(
                        text = eyebrow,
                        style = homeEyebrowStyle(),
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                    )
                    Spacer(Modifier.height(IndelibleSpacing.step4))
                }
                Text(
                    text = item.title,
                    style = MaterialTheme.typography.titleLarge,
                    color = MaterialTheme.colorScheme.onSurface,
                    maxLines = 2,
                    overflow = TextOverflow.Ellipsis,
                )
                if (!item.excerpt.isNullOrBlank()) {
                    Spacer(Modifier.height(IndelibleSpacing.step2))
                    Text(
                        text = item.excerpt,
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                        maxLines = 2,
                        overflow = TextOverflow.Ellipsis,
                    )
                }
                if (item.progressPercent != null) {
                    Spacer(Modifier.height(IndelibleSpacing.step8))
                    RecentlySavedProgressBar(
                        progress = item.progressFraction,
                        modifier = Modifier.width(IndelibleSpacing.step80 + IndelibleSpacing.step40),
                    )
                }
            }
        }
        if (showDivider) {
            HorizontalDivider(
                color = MaterialTheme.colorScheme.outlineVariant,
                modifier = Modifier.padding(start = IndelibleSpacing.rowPaddingH),
            )
        }
    }
}

@Composable
private fun recentlySavedEyebrow(item: HomeItem): String =
    buildString {
        item.domain?.takeIf { it.isNotBlank() }?.let { append(it.uppercase()) }
        if (!item.domain.isNullOrBlank() && item.readingTimeMinutes != null) append(" · ")
        item.readingTimeMinutes?.let {
            append(pluralStringResource(Res.plurals.home_reading_minutes, it, it))
        }
    }

@Composable
private fun RecentlySavedProgressBar(
    progress: Float,
    modifier: Modifier = Modifier,
) {
    Box(
        modifier =
            modifier
                .height(IndelibleSpacing.step4)
                .clip(IndelibleShape.xs)
                .background(MaterialTheme.colorScheme.outlineVariant),
    ) {
        Box(
            modifier =
                Modifier
                    .fillMaxWidth(fraction = progress.coerceIn(0f, 1f))
                    .matchParentSize()
                    .clip(IndelibleShape.xs)
                    .background(MaterialTheme.colorScheme.primary),
        )
    }
}

private val previewRead = Instant.parse("2026-05-20T08:00:00Z")

@Preview(showBackground = true)
@Composable
private fun RecentlySavedRowPreviewLight() {
    AppTheme(darkTheme = false) {
        Surface {
            Column {
                RecentlySavedRow(
                    item = previewHomeItem(id = "r1", progressPercent = null),
                    onClick = {},
                )
                RecentlySavedRow(
                    item =
                        previewHomeItem(
                            id = "r2",
                            title = "Notes on Slow Software",
                            domain = "increment.com",
                            progressPercent = 40f,
                        ).copy(lastReadAt = previewRead),
                    onClick = {},
                    showDivider = false,
                )
            }
        }
    }
}

@Preview(showBackground = true, uiMode = 0x20)
@Composable
private fun RecentlySavedRowPreviewDark() {
    AppTheme(darkTheme = true) {
        Surface {
            Column {
                RecentlySavedRow(
                    item = previewHomeItem(id = "r3", progressPercent = null),
                    onClick = {},
                )
                RecentlySavedRow(
                    item =
                        previewHomeItem(
                            id = "r4",
                            title = "Notes on Slow Software",
                            domain = "increment.com",
                            progressPercent = 40f,
                        ).copy(lastReadAt = previewRead),
                    onClick = {},
                    showDivider = false,
                )
            }
        }
    }
}
