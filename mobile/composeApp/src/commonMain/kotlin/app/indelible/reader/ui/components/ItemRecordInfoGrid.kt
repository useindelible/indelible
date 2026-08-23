package app.indelible.reader.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.unit.dp
import app.indelible.core.i18n.LocaleFormatters
import app.indelible.core.i18n.LocalizedDateStyle
import app.indelible.reader.model.ReaderDocument
import app.indelible.ui.theme.IndelibleShape
import app.indelible.ui.theme.IndelibleSpacing
import app.indelible.ui.theme.IndelibleTheme
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.reader_info_length
import indelible.composeapp.generated.resources.reader_info_progress
import indelible.composeapp.generated.resources.reader_info_published
import indelible.composeapp.generated.resources.reader_info_saved
import indelible.composeapp.generated.resources.reader_info_source
import indelible.composeapp.generated.resources.reader_info_type
import indelible.composeapp.generated.resources.reader_minutes_short
import indelible.composeapp.generated.resources.reader_progress_percent
import indelible.composeapp.generated.resources.reader_type_article
import indelible.composeapp.generated.resources.reader_type_book
import indelible.composeapp.generated.resources.reader_type_pdf
import indelible.composeapp.generated.resources.reader_type_unknown
import indelible.composeapp.generated.resources.reader_type_video
import indelible.composeapp.generated.resources.reader_words_count
import org.jetbrains.compose.resources.StringResource
import org.jetbrains.compose.resources.pluralStringResource
import org.jetbrains.compose.resources.stringResource
import kotlin.math.roundToInt

/**
 * The item-record "Info" table: hairline-divided rows of mono keys against
 * right-aligned values, ending in the accent progress bar. Mirrors the
 * prototype `pii-grid`. Rendered inside [ItemRecordPanel]'s Info section.
 */
@Composable
internal fun InfoGrid(
    item: ReaderDocument,
    progress: Float,
) {
    val accent = MaterialTheme.colorScheme.primary
    Column(modifier = Modifier.fillMaxWidth()) {
        InfoDivider()
        item.domain?.takeIf { it.isNotBlank() }?.let { domain ->
            InfoRow(stringResource(Res.string.reader_info_source)) { InfoValueText(domain, color = accent) }
            InfoDivider()
        }
        InfoRow(stringResource(Res.string.reader_info_type)) {
            InfoValueText(stringResource(itemTypeLabelRes(item.itemType)))
        }
        InfoDivider()
        item.publishedAt?.let {
            InfoRow(stringResource(Res.string.reader_info_published)) {
                InfoValueText(LocaleFormatters.date(it, LocalizedDateStyle.MEDIUM))
            }
            InfoDivider()
        }
        InfoRow(stringResource(Res.string.reader_info_saved)) {
            InfoValueText(LocaleFormatters.date(item.savedAt, LocalizedDateStyle.MEDIUM))
        }
        InfoDivider()
        lengthValue(item)?.let { (lead, mut) ->
            InfoRow(stringResource(Res.string.reader_info_length)) { LengthValue(lead = lead, mut = mut) }
            InfoDivider()
        }
        InfoRow(stringResource(Res.string.reader_info_progress)) { ProgressValue(progress) }
        InfoDivider()
    }
}

@Composable
private fun InfoDivider() {
    HorizontalDivider(thickness = 1.dp, color = MaterialTheme.colorScheme.outlineVariant)
}

@Composable
private fun InfoRow(
    label: String,
    value: @Composable () -> Unit,
) {
    Row(
        modifier =
            Modifier
                .fillMaxWidth()
                .padding(vertical = IndelibleSpacing.step12),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Text(
            text = label,
            style = monoLabelStyle(),
            color = IndelibleTheme.colors.textTertiary,
        )
        value()
    }
}

@Composable
private fun infoValueStyle() = MaterialTheme.typography.bodyMedium.copy(fontWeight = FontWeight.Medium)

@Composable
private fun InfoValueText(
    text: String,
    color: Color = MaterialTheme.colorScheme.onSurfaceVariant,
) {
    Text(text = text, style = infoValueStyle(), color = color, textAlign = TextAlign.End)
}

@Composable
private fun LengthValue(
    lead: String,
    mut: String?,
) {
    val tertiary = IndelibleTheme.colors.textTertiary
    val text =
        buildAnnotatedString {
            append(lead)
            if (mut != null) {
                withStyle(SpanStyle(color = tertiary, fontWeight = FontWeight.Normal)) {
                    append("  ·  $mut")
                }
            }
        }
    Text(
        text = text,
        style = infoValueStyle(),
        color = MaterialTheme.colorScheme.onSurfaceVariant,
        textAlign = TextAlign.End,
    )
}

private const val PERCENT_MAX = 100f

@Composable
private fun ProgressValue(progress: Float) {
    val fraction = (progress / PERCENT_MAX).coerceIn(0f, 1f)
    Row(
        horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step10),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Box(
            modifier =
                Modifier
                    .width(IndelibleSpacing.step80)
                    .height(IndelibleSpacing.step4)
                    .clip(IndelibleShape.xs)
                    .background(MaterialTheme.colorScheme.surfaceVariant),
        ) {
            Box(
                modifier =
                    Modifier
                        .fillMaxHeight()
                        .fillMaxWidth(fraction)
                        .clip(IndelibleShape.xs)
                        .background(MaterialTheme.colorScheme.primary),
            )
        }
        Text(
            text = stringResource(Res.string.reader_progress_percent, progress.roundToInt()),
            style = infoValueStyle(),
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
    }
}

/** Splits the length metadata into a leading "N min" and a muted "X words" tail. */
@Composable
private fun lengthValue(item: ReaderDocument): Pair<String, String?>? {
    val minutes =
        item.readingTimeMinutes?.let {
            pluralStringResource(Res.plurals.reader_minutes_short, it, it)
        }
    val words =
        item.wordCount?.let {
            pluralStringResource(
                Res.plurals.reader_words_count,
                it,
                LocaleFormatters.number(it.toLong()),
            )
        }
    return when {
        minutes != null -> minutes to words
        words != null -> words to null
        else -> null
    }
}

private fun itemTypeLabelRes(itemType: String): StringResource =
    when (itemType.lowercase()) {
        "article" -> Res.string.reader_type_article
        "video" -> Res.string.reader_type_video
        "pdf" -> Res.string.reader_type_pdf
        "book", "epub" -> Res.string.reader_type_book
        else -> Res.string.reader_type_unknown
    }
