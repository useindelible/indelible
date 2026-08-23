package app.indelible.feed.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.border
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
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Check
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.em
import app.indelible.core.model.ThumbnailColor
import app.indelible.core.i18n.relativeTimeText
import app.indelible.feed.model.FeedItemWithState
import app.indelible.ui.platform.rememberHapticTick
import app.indelible.ui.theme.AppTheme
import app.indelible.ui.theme.IndelibleIcons
import app.indelible.ui.theme.IndelibleShape
import app.indelible.ui.theme.IndelibleSpacing
import app.indelible.ui.theme.IndelibleTheme
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.feed_save
import indelible.composeapp.generated.resources.feed_saved
import indelible.composeapp.generated.resources.feed_seen
import kotlinx.datetime.Clock
import kotlinx.datetime.Instant
import org.jetbrains.compose.resources.stringResource

private const val STATE_SEEN = "seen"
private const val STATE_UNSEEN = "unseen"
private const val SEEN_DIM_ALPHA = 0.5f

/**
 * A feed list row matching the library row's shape: a large placeholder cover (with
 * an unread dot while unseen), an uppercase source/time eyebrow, title, summary, and
 * a Save pill that flips to a non-interactive "Saved" done-state once the item is in
 * the library. Seen rows render at half opacity, mirroring the prototype. Feed items
 * carry no media type, duration, or image, so the library's type badge, play overlay,
 * and thumbnail artwork are intentionally absent.
 */
@Composable
fun FeedItemRow(
    item: FeedItemWithState,
    saved: Boolean,
    onSave: () -> Unit,
    modifier: Modifier = Modifier,
    onOpen: () -> Unit = {},
    showDivider: Boolean = true,
) {
    val rowAlpha = if (item.state == STATE_SEEN) SEEN_DIM_ALPHA else 1f
    Column(modifier = modifier.background(MaterialTheme.colorScheme.surface)) {
        Row(
            modifier =
                Modifier
                    .fillMaxWidth()
                    .clickable(onClick = onOpen)
                    .alpha(rowAlpha)
                    .padding(
                        horizontal = IndelibleSpacing.rowPaddingH,
                        vertical = IndelibleSpacing.step16,
                    ),
            verticalAlignment = Alignment.Top,
        ) {
            FeedThumbnail(
                item = item,
                modifier = Modifier.size(IndelibleSpacing.step96),
            )
            Spacer(Modifier.width(IndelibleSpacing.rowContentGap))
            Column(modifier = Modifier.weight(1f)) {
                Text(
                    text = feedEyebrow(item),
                    style = feedEyebrowStyle(),
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
                Spacer(Modifier.height(IndelibleSpacing.step4))
                Text(
                    text = item.title,
                    style = MaterialTheme.typography.titleLarge,
                    color = MaterialTheme.colorScheme.onSurface,
                    maxLines = 2,
                    overflow = TextOverflow.Ellipsis,
                )
                val excerpt = item.excerpt
                if (!excerpt.isNullOrBlank()) {
                    Spacer(Modifier.height(IndelibleSpacing.step2))
                    Text(
                        text = excerpt,
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                        maxLines = 2,
                        overflow = TextOverflow.Ellipsis,
                    )
                }
                Spacer(Modifier.height(IndelibleSpacing.step10))
                FeedSaveButton(saved = saved, onSave = onSave)
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
private fun FeedThumbnail(
    item: FeedItemWithState,
    modifier: Modifier = Modifier,
) {
    val palette = IndelibleTheme.colors.collectionBanners
    val placeholder = palette[ThumbnailColor.forId(item.id).ordinal % palette.size]
    Box(modifier = modifier) {
        Box(
            modifier =
                Modifier
                    .matchParentSize()
                    .clip(IndelibleShape.xl)
                    .background(placeholder),
            contentAlignment = Alignment.Center,
        ) {
            Text(
                text =
                    item.title
                        .firstOrNull()
                        ?.uppercaseChar()
                        ?.toString() ?: "?",
                style = MaterialTheme.typography.headlineSmall,
                color = MaterialTheme.colorScheme.onSurface,
            )
        }
        if (item.state == STATE_UNSEEN) {
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
}

@Composable
private fun FeedSaveButton(
    saved: Boolean,
    onSave: () -> Unit,
) {
    val hapticTick = rememberHapticTick()
    if (saved) {
        Row(
            modifier =
                Modifier
                    .clip(IndelibleShape.full)
                    .background(MaterialTheme.colorScheme.primaryContainer)
                    .padding(horizontal = IndelibleSpacing.step12, vertical = IndelibleSpacing.step6),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Icon(
                imageVector = Icons.Filled.Check,
                contentDescription = null,
                tint = MaterialTheme.colorScheme.primary,
                modifier = Modifier.size(IndelibleSpacing.step16),
            )
            Spacer(Modifier.width(IndelibleSpacing.step4))
            Text(
                text = stringResource(Res.string.feed_saved),
                style = MaterialTheme.typography.titleSmall,
                color = MaterialTheme.colorScheme.primary,
            )
        }
    } else {
        Row(
            modifier =
                Modifier
                    .clip(IndelibleShape.full)
                    .border(
                        width = IndelibleSpacing.step2 / 2,
                        color = MaterialTheme.colorScheme.outline,
                        shape = IndelibleShape.full,
                    ).clickable(onClick = {
                        hapticTick()
                        onSave()
                    })
                    .padding(horizontal = IndelibleSpacing.step12, vertical = IndelibleSpacing.step6),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Icon(
                imageVector = IndelibleIcons.Plus,
                contentDescription = null,
                tint = MaterialTheme.colorScheme.onSurfaceVariant,
                modifier = Modifier.size(IndelibleSpacing.step16),
            )
            Spacer(Modifier.width(IndelibleSpacing.step4))
            Text(
                text = stringResource(Res.string.feed_save),
                style = MaterialTheme.typography.titleSmall,
                color = MaterialTheme.colorScheme.onSurface,
            )
        }
    }
}

@Composable
private fun feedEyebrow(item: FeedItemWithState): String {
    val now = currentInstant
    return buildString {
        feedDomain(item.url)?.let { append(it.uppercase()) }
        val timestamp = item.publishedAt ?: item.fetchedAt
        if (isNotEmpty()) append(" · ")
        append(relativeTimeText(timestamp, now).uppercase())
        if (item.state == STATE_SEEN) {
            append(" · ")
            append(stringResource(Res.string.feed_seen).uppercase())
        }
    }
}

private fun feedDomain(url: String?): String? {
    val domain =
        url
            ?.removePrefix("https://")
            ?.removePrefix("http://")
            ?.removePrefix("www.")
            ?.substringBefore("/")
    return domain?.takeIf { it.isNotBlank() }
}

private val currentInstant: Instant
    get() = Clock.System.now()

/**
 * Caption-1 treatment (uppercase eyebrows): wide-tracked medium `labelSmall`,
 * matching the library row eyebrow so the two lists read identically.
 */
@Composable
private fun feedEyebrowStyle(): TextStyle =
    MaterialTheme.typography.labelSmall.copy(
        fontWeight = FontWeight.Medium,
        letterSpacing = 0.06.em,
    )

@Preview(showBackground = true)
@Composable
private fun FeedItemRowPreviewLight() {
    AppTheme(darkTheme = false) {
        Surface {
            Column {
                FeedItemRow(
                    item = previewFeedItem(id = "f1"),
                    saved = false,
                    onSave = {},
                )
                FeedItemRow(
                    item = previewFeedItem(id = "f2", state = "seen"),
                    saved = true,
                    onSave = {},
                    showDivider = false,
                )
            }
        }
    }
}

@Preview(showBackground = true, uiMode = 0x20)
@Composable
private fun FeedItemRowPreviewDark() {
    AppTheme(darkTheme = true) {
        Surface {
            Column {
                FeedItemRow(
                    item = previewFeedItem(id = "f3"),
                    saved = true,
                    onSave = {},
                )
                FeedItemRow(
                    item = previewFeedItem(id = "f4", state = "seen"),
                    saved = false,
                    onSave = {},
                    showDivider = false,
                )
            }
        }
    }
}

private fun previewFeedItem(
    id: String,
    state: String = "unseen",
) = FeedItemWithState(
    id = id,
    guid = "entry-$id",
    subscriptionId = "sub-1",
    sourceId = "src-1",
    title = "Understanding Compose Multiplatform Navigation",
    url = "https://blog.jetbrains.com/kotlin/compose-nav",
    author = "JetBrains",
    excerpt = "A guide to setting up navigation in KMP apps without losing your mind across two platforms.",
    publishedAt = Instant.parse("2026-03-28T10:00:00Z"),
    fetchedAt = Instant.parse("2026-03-28T12:00:00Z"),
    saved = false,
    state = state,
)
