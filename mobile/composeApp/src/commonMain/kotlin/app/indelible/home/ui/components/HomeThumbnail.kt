package app.indelible.home.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.em
import app.indelible.core.model.ThumbnailColor
import app.indelible.home.model.HomeItem
import app.indelible.ui.theme.IndelibleTheme
import coil3.compose.AsyncImage
import kotlinx.datetime.Instant

/**
 * Caption-1 styling (uppercase section eyebrows / kickers): wide-tracked medium
 * labelSmall, per the call-site pattern documented in Type.kt. Apply uppercase to
 * the text itself.
 */
@Composable
internal fun homeEyebrowStyle(): TextStyle =
    MaterialTheme.typography.labelSmall.copy(
        fontWeight = FontWeight.Medium,
        letterSpacing = 0.06.em,
    )

/**
 * Shared image-or-placeholder tile for home cards. When the item has no artwork,
 * a deterministic pastel from the collection-banner palette stands in, keyed off
 * the item id so a given item always shows the same colour.
 */
@Composable
internal fun HomeThumbnail(
    item: HomeItem,
    shape: Shape,
    modifier: Modifier = Modifier,
) {
    val image = item.thumbnailUrl ?: item.leadImageUrl
    Box(modifier = modifier.clip(shape)) {
        if (image != null) {
            AsyncImage(
                model = image,
                contentDescription = null,
                contentScale = ContentScale.Crop,
                modifier = Modifier.matchParentSize(),
            )
        } else {
            val palette = IndelibleTheme.colors.collectionBanners
            val placeholder = palette[ThumbnailColor.forId(item.id).ordinal % palette.size]
            Box(
                modifier = Modifier.matchParentSize().background(placeholder),
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
}

private val previewInstant = Instant.parse("2026-05-24T08:00:00Z")

internal fun previewHomeItem(
    id: String = "lib_1",
    title: String = "The Quiet Architecture of Attention",
    itemType: String = "article",
    domain: String? = "newyorker.com",
    excerpt: String? =
        "How the spaces between ideas shape the way we read, and why the best long-form" +
            " writing leaves room to think.",
    author: String? = "Maya Lindqvist",
    readingTimeMinutes: Int? = 12,
    progressPercent: Float? = 62f,
    leadImageUrl: String? = null,
): HomeItem =
    HomeItem(
        id = id,
        title = title,
        itemType = itemType,
        createdAt = previewInstant,
        domain = domain,
        excerpt = excerpt,
        author = author,
        readingTimeMinutes = readingTimeMinutes,
        progressPercent = progressPercent,
        leadImageUrl = leadImageUrl,
    )
