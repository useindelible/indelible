package app.indelible.search.ui.components

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
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.style.TextOverflow
import app.indelible.search.model.SearchResult
import app.indelible.ui.theme.IndelibleShape
import app.indelible.ui.theme.IndelibleSpacing
import app.indelible.ui.theme.IndelibleTheme
import kotlinx.datetime.Clock
import kotlinx.datetime.Instant
import kotlinx.datetime.TimeZone
import kotlinx.datetime.toLocalDateTime

@Composable
fun SearchResultRow(
    result: SearchResult,
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
            ContentTypeThumbnail(
                contentType = result.contentType,
                modifier = Modifier.size(IndelibleSpacing.step48),
            )
            Spacer(modifier = Modifier.width(IndelibleSpacing.step14))
            Column(modifier = Modifier.weight(1f)) {
                Text(
                    text = result.title,
                    style = MaterialTheme.typography.titleSmall,
                    color = MaterialTheme.colorScheme.onSurface,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
                if (result.snippet.isNotBlank()) {
                    val highlightBg = MaterialTheme.colorScheme.primary.copy(alpha = 0.15f)
                    Text(
                        text = parseSnippetHtml(result.snippet, highlightBg),
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                        maxLines = 2,
                        overflow = TextOverflow.Ellipsis,
                    )
                }
                val metaText = buildMeta(result)
                if (metaText.isNotBlank()) {
                    Text(
                        text = metaText,
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                        modifier = Modifier.padding(top = IndelibleSpacing.step2),
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
private fun ContentTypeThumbnail(
    contentType: String,
    modifier: Modifier = Modifier,
) {
    val bgColor = contentTypeBg(contentType)
    Box(
        modifier =
            modifier
                .clip(IndelibleShape.md)
                .background(bgColor),
        contentAlignment = Alignment.Center,
    ) {
        Text(
            text = contentTypeEmoji(contentType),
            style = MaterialTheme.typography.titleLarge,
        )
    }
}

@Composable
private fun contentTypeBg(contentType: String): Color {
    val primary = MaterialTheme.colorScheme.primary
    val error = MaterialTheme.colorScheme.error
    val success = IndelibleTheme.colors.success
    val warning = IndelibleTheme.colors.warning
    return when (contentType) {
        "article" -> primary.copy(alpha = 0.12f)
        "book" -> success.copy(alpha = 0.12f)
        "pdf" -> error.copy(alpha = 0.12f)
        "video" -> warning.copy(alpha = 0.12f)
        "email" -> warning.copy(alpha = 0.10f)
        "tweet" -> primary.copy(alpha = 0.08f)
        "podcast" -> MaterialTheme.colorScheme.primaryContainer.copy(alpha = 0.5f)
        else -> MaterialTheme.colorScheme.surfaceVariant
    }
}

private fun contentTypeEmoji(contentType: String): String =
    when (contentType) {
        "article" -> "\uD83D\uDCF0"
        "book" -> "\uD83D\uDCD6"
        "pdf" -> "\uD83D\uDCC4"
        "video" -> "\uD83C\uDFAC"
        "email" -> "\u2709\uFE0F"
        "tweet" -> "\uD83D\uDC26"
        "podcast" -> "\uD83C\uDFA7"
        else -> "\uD83D\uDCF0"
    }

private fun buildMeta(result: SearchResult): String {
    val domain =
        result.url?.let {
            runCatching {
                // Extract host from URL without using java.net.URL (not available on iOS)
                it.substringAfter("://").substringBefore("/").removePrefix("www.")
            }.getOrNull()
        }
    val date = formatRelativeDate(result.savedAt)
    return listOfNotNull(domain, date).joinToString(" · ")
}

private const val DAYS_PER_WEEK = 7
private const val DAYS_PER_MONTH = 30
private const val MONTH_ABBREVIATION_LENGTH = 3

private fun formatRelativeDate(instant: Instant?): String {
    if (instant == null) return ""
    return runCatching {
        val diff = Clock.System.now() - instant
        val diffDays = diff.inWholeDays
        when {
            diffDays == 0L ->
                if (diff.inWholeHours == 0L) "${diff.inWholeMinutes}m" else "${diff.inWholeHours}h"
            diffDays < DAYS_PER_WEEK -> "${diffDays}d"
            diffDays < DAYS_PER_MONTH -> "${diffDays / DAYS_PER_WEEK}w"
            else -> {
                val ld = instant.toLocalDateTime(TimeZone.currentSystemDefault())
                val month =
                    ld.month.name
                        .lowercase()
                        .replaceFirstChar { it.uppercase() }
                        .take(MONTH_ABBREVIATION_LENGTH)
                "$month ${ld.dayOfMonth}"
            }
        }
    }.getOrElse { "" }
}
