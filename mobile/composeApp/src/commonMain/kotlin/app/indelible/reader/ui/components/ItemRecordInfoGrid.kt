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
import app.indelible.reader.model.ReaderDocument
import app.indelible.ui.theme.IndelibleShape
import app.indelible.ui.theme.IndelibleSpacing
import app.indelible.ui.theme.IndelibleTheme
import kotlin.math.roundToInt
import kotlinx.datetime.Instant
import kotlinx.datetime.TimeZone
import kotlinx.datetime.toLocalDateTime

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
            InfoRow("Source") { InfoValueText(domain, color = accent) }
            InfoDivider()
        }
        InfoRow("Type") { InfoValueText(item.itemType.replaceFirstChar { it.uppercaseChar() }) }
        InfoDivider()
        item.publishedAt?.let {
            InfoRow("Published") { InfoValueText(formatRecordDate(it)) }
            InfoDivider()
        }
        InfoRow("Saved") { InfoValueText(formatRecordDate(item.savedAt)) }
        InfoDivider()
        lengthValue(item)?.let { (lead, mut) ->
            InfoRow("Length") { LengthValue(lead = lead, mut = mut) }
            InfoDivider()
        }
        InfoRow("Progress") { ProgressValue(progress) }
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
            text = label.uppercase(),
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
private const val THOUSANDS_GROUP_SIZE = 3

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
            text = "${progress.roundToInt()}%",
            style = infoValueStyle(),
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
    }
}

private val MONTH_ABBREVIATIONS =
    listOf("Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec")

private fun formatRecordDate(instant: Instant): String {
    val dt = instant.toLocalDateTime(TimeZone.currentSystemDefault())
    val month = MONTH_ABBREVIATIONS.getOrElse(dt.monthNumber - 1) { "" }
    return "$month ${dt.dayOfMonth}, ${dt.year}"
}

/** Splits the length metadata into a leading "N min" and a muted "X words" tail. */
private fun lengthValue(item: ReaderDocument): Pair<String, String?>? {
    val minutes = item.readingTimeMinutes?.let { "$it min" }
    val words = item.wordCount?.let { "${it.withThousands()} words" }
    return when {
        minutes != null -> minutes to words
        words != null -> words to null
        else -> null
    }
}

private fun Int.withThousands(): String {
    val digits = toString()
    return buildString {
        for (i in digits.indices) {
            if (i > 0 && (digits.length - i) % THOUSANDS_GROUP_SIZE == 0) append(',')
            append(digits[i])
        }
    }
}
