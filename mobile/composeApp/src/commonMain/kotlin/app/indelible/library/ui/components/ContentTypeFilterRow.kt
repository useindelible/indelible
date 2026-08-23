package app.indelible.library.ui.components

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.defaultMinSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.tooling.preview.Preview
import app.indelible.core.i18n.LocaleFormatters
import app.indelible.core.model.LibraryCounts
import app.indelible.library.viewmodel.ContentTypeFilter
import app.indelible.ui.theme.AppTheme
import app.indelible.ui.theme.IndelibleShape
import app.indelible.ui.theme.IndelibleSpacing
import app.indelible.ui.theme.geistMonoFontFamily
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.library_content_all
import indelible.composeapp.generated.resources.library_content_articles
import indelible.composeapp.generated.resources.library_content_books
import indelible.composeapp.generated.resources.library_content_emails
import indelible.composeapp.generated.resources.library_content_pdfs
import indelible.composeapp.generated.resources.library_content_podcasts
import indelible.composeapp.generated.resources.library_content_tweets
import indelible.composeapp.generated.resources.library_content_videos
import org.jetbrains.compose.resources.StringResource
import org.jetbrains.compose.resources.stringResource

private const val CHIP_COUNT_ALPHA = 0.72f

/**
 * Content-type filter chips (prototype `.filters`): All plus every type the scope
 * actually holds, each carrying its count. Types with nothing saved are omitted so the
 * row reflects the library rather than the enum, but an active selectable filter stays
 * visible — otherwise selecting a type would make its own chip disappear.
 */
@Composable
fun ContentTypeFilterRow(
    selected: ContentTypeFilter,
    counts: LibraryCounts?,
    onSelect: (ContentTypeFilter) -> Unit,
    modifier: Modifier = Modifier,
) {
    val filters = visibleContentTypeFilters(selected, counts)

    LazyRow(
        modifier = modifier,
        contentPadding = PaddingValues(horizontal = IndelibleSpacing.rowPaddingH),
        horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step8),
    ) {
        items(filters) { filter ->
            ContentTypeChip(
                filter = filter,
                selected = filter == selected,
                count = counts?.countFor(filter),
                onSelect = { onSelect(filter) },
            )
        }
    }
}

/**
 * All + the selectable types present in [counts], preserving enum order. With no counts
 * loaded the selectable set shows rather than collapsing to a lone chip.
 */
internal fun visibleContentTypeFilters(
    selected: ContentTypeFilter,
    counts: LibraryCounts?,
): List<ContentTypeFilter> {
    val selectableFilters = ContentTypeFilter.entries.filterNot { it == ContentTypeFilter.PODCASTS }
    if (counts == null) return selectableFilters
    if (counts.total == 0) return selectableFilters
    return selectableFilters.filter { filter ->
        filter == ContentTypeFilter.ALL || filter == selected || counts.countFor(filter) > 0
    }
}

internal fun LibraryCounts.countFor(filter: ContentTypeFilter): Int =
    when (val apiValue = filter.apiValue) {
        null -> total
        else -> byItemType[apiValue] ?: 0
    }

@Composable
private fun ContentTypeChip(
    filter: ContentTypeFilter,
    selected: Boolean,
    count: Int?,
    onSelect: () -> Unit,
) {
    val contentColor =
        if (selected) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onSurfaceVariant
    val background =
        if (selected) MaterialTheme.colorScheme.primaryContainer else MaterialTheme.colorScheme.surfaceContainerHigh
    val borderColor = if (selected) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.outlineVariant

    Row(
        modifier =
            Modifier
                .defaultMinSize(minHeight = IndelibleSpacing.step32)
                .clip(IndelibleShape.sm)
                .background(background)
                .border(BorderStroke(IndelibleSpacing.hairline, borderColor), IndelibleShape.sm)
                .clickable(onClick = onSelect)
                .padding(horizontal = IndelibleSpacing.step12),
        horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step6),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Text(
            text = stringResource(filter.labelRes),
            style = MaterialTheme.typography.bodyMedium.copy(fontWeight = FontWeight.Medium),
            color = contentColor,
        )
        if (count != null) {
            Text(
                text = LocaleFormatters.number(count.toLong()),
                style = MaterialTheme.typography.bodySmall.copy(fontFamily = geistMonoFontFamily()),
                color = contentColor.copy(alpha = CHIP_COUNT_ALPHA),
            )
        }
    }
}

private val ContentTypeFilter.labelRes: StringResource
    get() =
        when (this) {
            ContentTypeFilter.ALL -> Res.string.library_content_all
            ContentTypeFilter.ARTICLES -> Res.string.library_content_articles
            ContentTypeFilter.BOOKS -> Res.string.library_content_books
            ContentTypeFilter.PDFS -> Res.string.library_content_pdfs
            ContentTypeFilter.EMAILS -> Res.string.library_content_emails
            ContentTypeFilter.TWEETS -> Res.string.library_content_tweets
            ContentTypeFilter.VIDEOS -> Res.string.library_content_videos
            ContentTypeFilter.PODCASTS -> Res.string.library_content_podcasts
        }

@Preview(showBackground = true)
@Composable
private fun ContentTypeFilterRowPreviewLight() {
    AppTheme(darkTheme = false) {
        Surface {
            ContentTypeFilterRow(
                selected = ContentTypeFilter.ALL,
                counts =
                    LibraryCounts(
                        total = 66,
                        unread = 42,
                        reading = 5,
                        done = 19,
                        byItemType = mapOf("article" to 38, "video" to 11, "pdf" to 9, "email" to 8),
                    ),
                onSelect = {},
            )
        }
    }
}

@Preview(showBackground = true, uiMode = 0x20)
@Composable
private fun ContentTypeFilterRowPreviewDark() {
    AppTheme(darkTheme = true) {
        Surface {
            ContentTypeFilterRow(
                selected = ContentTypeFilter.ARTICLES,
                counts =
                    LibraryCounts(
                        total = 66,
                        unread = 42,
                        reading = 5,
                        done = 19,
                        byItemType = mapOf("article" to 38, "video" to 11, "pdf" to 9, "email" to 8),
                    ),
                onSelect = {},
            )
        }
    }
}
