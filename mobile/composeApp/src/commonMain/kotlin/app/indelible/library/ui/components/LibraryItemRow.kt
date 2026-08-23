package app.indelible.library.ui.components

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
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.PlayArrow
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.em
import app.indelible.core.model.LibraryItem
import app.indelible.core.model.ThumbnailColor
import app.indelible.core.model.readingMinutesLeft
import app.indelible.ui.theme.AppTheme
import app.indelible.ui.theme.IndelibleShape
import app.indelible.ui.theme.IndelibleSpacing
import app.indelible.ui.theme.IndelibleTheme
import coil3.compose.AsyncImage
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.library_content_books
import indelible.composeapp.generated.resources.library_content_emails
import indelible.composeapp.generated.resources.library_content_pdfs
import indelible.composeapp.generated.resources.library_content_podcasts
import indelible.composeapp.generated.resources.library_content_tweets
import indelible.composeapp.generated.resources.library_content_videos
import indelible.composeapp.generated.resources.library_new
import indelible.composeapp.generated.resources.library_progress_minutes_left
import indelible.composeapp.generated.resources.library_progress_percent
import indelible.composeapp.generated.resources.library_reading_time
import kotlinx.datetime.Instant
import org.jetbrains.compose.resources.StringResource
import org.jetbrains.compose.resources.pluralStringResource
import org.jetbrains.compose.resources.stringResource
import kotlin.math.roundToInt

private const val MEDIA_ARTICLE = "article"
private const val MEDIA_VIDEO = "video"
private const val MEDIA_PODCAST = "podcast"

/**
 * A library list row: large cover (with type badge, media play/duration, and an
 * unread dot when relevant), an uppercase source/length eyebrow that leads with
 * an accent "New" when unread, title, summary, and a progress sliver when
 * partially read. Shares its layout with the home "Recently saved" row so the two
 * lists read identically.
 */
@Composable
fun LibraryItemRow(
    item: LibraryItem,
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
                        vertical = IndelibleSpacing.step16,
                    ),
            verticalAlignment = Alignment.Top,
        ) {
            LibraryThumbnail(
                item = item,
                modifier = Modifier.size(IndelibleSpacing.step96),
            )
            Spacer(Modifier.width(IndelibleSpacing.rowContentGap))
            Column(modifier = Modifier.weight(1f)) {
                LibraryRowEyebrow(item)
                Text(
                    text = item.title,
                    style = MaterialTheme.typography.titleLarge,
                    color = MaterialTheme.colorScheme.onSurface,
                    maxLines = 2,
                    overflow = TextOverflow.Ellipsis,
                )
                val summary = item.summary ?: item.excerpt
                if (!summary.isNullOrBlank()) {
                    Spacer(Modifier.height(IndelibleSpacing.step2))
                    Text(
                        text = summary,
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                        maxLines = 2,
                        overflow = TextOverflow.Ellipsis,
                    )
                }
                item.progressPercent?.let { progress ->
                    Spacer(Modifier.height(IndelibleSpacing.step8))
                    LibraryProgress(progress = progress, minutesLeft = item.readingMinutesLeft())
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
private fun LibraryRowEyebrow(item: LibraryItem) {
    val accent = MaterialTheme.colorScheme.primary
    val muted = MaterialTheme.colorScheme.onSurfaceVariant
    val domain = item.domain?.takeIf { it.isNotBlank() }
    val readingTime = item.readingTimeMinutes
    val newLabel = stringResource(Res.string.library_new)
    val readingTimeLabel = readingTime?.let { pluralStringResource(Res.plurals.library_reading_time, it, it) }

    if (!item.isUnread && domain == null && readingTime == null) return

    val eyebrow =
        buildAnnotatedString {
            var needSeparator = false
            if (item.isUnread) {
                withStyle(SpanStyle(color = accent)) { append(newLabel) }
                needSeparator = true
            }
            if (domain != null) {
                if (needSeparator) append(" · ")
                append(domain.uppercase())
                needSeparator = true
            }
            if (readingTimeLabel != null) {
                if (needSeparator) append(" · ")
                append(readingTimeLabel)
            }
        }

    Text(
        text = eyebrow,
        style = libraryEyebrowStyle(),
        color = muted,
        maxLines = 1,
        overflow = TextOverflow.Ellipsis,
    )
    Spacer(Modifier.height(IndelibleSpacing.step4))
}

@Composable
private fun LibraryThumbnail(
    item: LibraryItem,
    modifier: Modifier = Modifier,
) {
    val isMedia = item.itemType == MEDIA_VIDEO || item.itemType == MEDIA_PODCAST

    Box(modifier = modifier) {
        Box(modifier = Modifier.matchParentSize().clip(IndelibleShape.xl)) {
            ThumbnailImageContent(item = item, modifier = Modifier.matchParentSize())
        }

        if (item.itemType != MEDIA_ARTICLE) {
            ItemTypeBadge(
                itemType = item.itemType,
                modifier =
                    Modifier
                        .align(Alignment.TopStart)
                        .padding(IndelibleSpacing.step6),
            )
        }

        if (isMedia) {
            PlayOverlay(modifier = Modifier.align(Alignment.Center))
            item.videoDurationSeconds?.let { seconds ->
                DurationBadge(
                    text = formatDuration(seconds),
                    modifier =
                        Modifier
                            .align(Alignment.BottomEnd)
                            .padding(IndelibleSpacing.step6),
                )
            }
        }

        if (item.isUnread) {
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
private fun ThumbnailImageContent(
    item: LibraryItem,
    modifier: Modifier,
) {
    val image = item.thumbnailUrl ?: item.leadImageUrl
    if (image != null) {
        AsyncImage(
            model = image,
            contentDescription = null,
            contentScale = ContentScale.Crop,
            modifier = modifier,
        )
    } else {
        val palette = IndelibleTheme.colors.collectionBanners
        val placeholder = palette[ThumbnailColor.forId(item.id).ordinal % palette.size]
        Box(
            modifier = modifier.background(placeholder),
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
    }
}

@Composable
private fun ItemTypeBadge(
    itemType: String,
    modifier: Modifier = Modifier,
) {
    val labelRes = itemTypeLabelRes(itemType) ?: return
    Box(
        modifier =
            modifier
                .clip(IndelibleShape.xs)
                .background(MaterialTheme.colorScheme.scrim.copy(alpha = 0.55f))
                .padding(
                    horizontal = IndelibleSpacing.step6,
                    vertical = IndelibleSpacing.step2,
                ),
    ) {
        Text(
            text = stringResource(labelRes),
            style = libraryEyebrowStyle(),
            color = MaterialTheme.colorScheme.onPrimary,
        )
    }
}

@Composable
private fun PlayOverlay(modifier: Modifier = Modifier) {
    Box(
        modifier =
            modifier
                .size(IndelibleSpacing.step40)
                .clip(IndelibleShape.full)
                .background(MaterialTheme.colorScheme.scrim.copy(alpha = 0.45f))
                .border(
                    width = IndelibleSpacing.step2 / 2,
                    color = MaterialTheme.colorScheme.onPrimary.copy(alpha = 0.4f),
                    shape = IndelibleShape.full,
                ),
        contentAlignment = Alignment.Center,
    ) {
        Icon(
            imageVector = Icons.Filled.PlayArrow,
            contentDescription = null,
            tint = MaterialTheme.colorScheme.onPrimary,
            modifier = Modifier.size(IndelibleSpacing.step20),
        )
    }
}

@Composable
private fun DurationBadge(
    text: String,
    modifier: Modifier = Modifier,
) {
    Box(
        modifier =
            modifier
                .clip(IndelibleShape.xs)
                .background(MaterialTheme.colorScheme.scrim.copy(alpha = 0.6f))
                .padding(
                    horizontal = IndelibleSpacing.step4,
                    vertical = IndelibleSpacing.step2,
                ),
    ) {
        Text(
            text = text,
            style = MaterialTheme.typography.labelSmall,
            color = MaterialTheme.colorScheme.onPrimary,
        )
    }
}

@Composable
private fun LibraryProgress(
    progress: Float,
    minutesLeft: Int?,
    modifier: Modifier = Modifier,
) {
    Row(
        modifier = modifier.fillMaxWidth(),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step10),
    ) {
        LibraryProgressBar(
            progress = progress,
            modifier = Modifier.weight(1f),
        )
        Text(
            text = progressLabel(progress, minutesLeft),
            style = libraryEyebrowStyle(),
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            maxLines = 1,
            softWrap = false,
            overflow = TextOverflow.Clip,
        )
    }
}

private const val PERCENT_SCALE = 100
private const val SECONDS_PER_HOUR = 3600
private const val SECONDS_PER_MINUTE = 60

@Composable
private fun progressLabel(
    progress: Float,
    minutesLeft: Int?,
): String {
    val percent = (progress.coerceIn(0f, 1f) * PERCENT_SCALE).roundToInt()
    return if (minutesLeft != null) {
        pluralStringResource(Res.plurals.library_progress_minutes_left, minutesLeft, percent, minutesLeft)
    } else {
        stringResource(Res.string.library_progress_percent, percent)
    }
}

private fun itemTypeLabelRes(itemType: String): StringResource? =
    when (itemType) {
        "book" -> Res.string.library_content_books
        "email" -> Res.string.library_content_emails
        "pdf" -> Res.string.library_content_pdfs
        "podcast" -> Res.string.library_content_podcasts
        "tweet" -> Res.string.library_content_tweets
        "video" -> Res.string.library_content_videos
        else -> null
    }

@Composable
private fun LibraryProgressBar(
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

/**
 * Caption-1 treatment (uppercase eyebrows / badges): wide-tracked medium
 * `labelSmall`, per the call-site recipe documented in Type.kt.
 */
@Composable
private fun libraryEyebrowStyle(): TextStyle =
    MaterialTheme.typography.labelSmall.copy(
        fontWeight = FontWeight.Medium,
        letterSpacing = 0.06.em,
    )

private fun formatDuration(totalSeconds: Int): String {
    val hours = totalSeconds / SECONDS_PER_HOUR
    val minutes = (totalSeconds % SECONDS_PER_HOUR) / SECONDS_PER_MINUTE
    val seconds = totalSeconds % SECONDS_PER_MINUTE
    return if (hours > 0) {
        "$hours:${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}"
    } else {
        "$minutes:${seconds.toString().padStart(2, '0')}"
    }
}

@Preview(showBackground = true)
@Composable
private fun LibraryItemRowPreviewLight() {
    AppTheme(darkTheme = false) {
        Surface {
            Column {
                LibraryItemRow(item = previewItem(id = "a1"), onClick = {})
                LibraryItemRow(
                    item =
                        previewItem(
                            id = "b2",
                            itemType = "video",
                            title = "Apple Vision Pro 2: The Spatial Computing Reset",
                            domain = "youtube.com",
                        ),
                    onClick = {},
                )
                LibraryItemRow(
                    item = previewItem(id = "c3"),
                    onClick = {},
                    showDivider = false,
                )
            }
        }
    }
}

@Preview(showBackground = true, uiMode = 0x20)
@Composable
private fun LibraryItemRowPreviewDark() {
    AppTheme(darkTheme = true) {
        Surface {
            Column {
                LibraryItemRow(
                    item = previewItem(id = "d4"),
                    onClick = {},
                )
                LibraryItemRow(
                    item =
                        previewItem(
                            id = "e5",
                            itemType = "pdf",
                            title = "Attention Is All You Need — Annotated Edition",
                            domain = "arxiv.org",
                        ),
                    onClick = {},
                    showDivider = false,
                )
            }
        }
    }
}

private fun previewItem(
    id: String,
    itemType: String = "article",
    title: String = "The Future of Open-Source AI Models",
    domain: String? = "techcrunch.com",
) = LibraryItem(
    id = id,
    documentId = "doc_$id",
    itemType = itemType,
    triageState = "inbox",
    isFavorite = false,
    isShortlisted = false,
    title = title,
    excerpt = "A deep dive into what the next generation of open models will look like and who they serve.",
    domain = domain,
    author = "Sarah Chen",
    savedAt = Instant.parse("2024-01-15T12:00:00Z"),
    source = "url",
    createdAt = Instant.parse("2024-01-15T12:00:00Z"),
    updatedAt = Instant.parse("2024-01-15T12:00:00Z"),
)
