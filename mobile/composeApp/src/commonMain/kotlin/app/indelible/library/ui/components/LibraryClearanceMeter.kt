package app.indelible.library.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.size
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.em
import app.indelible.core.model.LibraryCounts
import app.indelible.ui.theme.AppTheme
import app.indelible.ui.theme.IndelibleShape
import app.indelible.ui.theme.IndelibleSpacing
import app.indelible.ui.theme.geistMonoFontFamily

private const val READING_SEGMENT_ALPHA = 0.42f

private data class MeterSegment(
    val label: String,
    val count: Int,
    val color: Color,
)

/**
 * Clearance meter (prototype `.meter` + `.meter-key`): the scope's unread/reading/done
 * split as one proportional bar plus a mono legend. Empty buckets are dropped rather
 * than drawn as slivers, and an empty scope renders nothing at all.
 */
@Composable
fun LibraryClearanceMeter(
    counts: LibraryCounts,
    modifier: Modifier = Modifier,
) {
    val segments =
        listOf(
            MeterSegment("unread", counts.unread, MaterialTheme.colorScheme.primary),
            MeterSegment(
                "reading",
                counts.reading,
                MaterialTheme.colorScheme.primary.copy(alpha = READING_SEGMENT_ALPHA),
            ),
            MeterSegment("done", counts.done, MaterialTheme.colorScheme.outlineVariant),
        ).filter { it.count > 0 }

    if (segments.isEmpty()) return

    val description = segments.joinToString(", ") { "${it.count} ${it.label}" }

    Column(
        modifier = modifier.fillMaxWidth(),
        verticalArrangement = Arrangement.spacedBy(IndelibleSpacing.step12),
    ) {
        Row(
            modifier =
                Modifier
                    .fillMaxWidth()
                    .height(IndelibleSpacing.step6)
                    .semantics { contentDescription = description },
            horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step4),
        ) {
            segments.forEach { segment ->
                Box(
                    modifier =
                        Modifier
                            .weight(segment.count.toFloat())
                            .fillMaxHeight()
                            .clip(IndelibleShape.xs)
                            .background(segment.color),
                )
            }
        }
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step14),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            segments.forEach { segment ->
                MeterLegendEntry(segment)
            }
        }
    }
}

@Composable
private fun MeterLegendEntry(segment: MeterSegment) {
    Row(
        horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step6),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Box(
            modifier =
                Modifier
                    .size(IndelibleSpacing.step6)
                    .clip(IndelibleShape.xs)
                    .background(segment.color),
        )
        Text(
            text = "${segment.count} ${segment.label}",
            style =
                MaterialTheme.typography.labelSmall.copy(
                    fontFamily = geistMonoFontFamily(),
                    fontWeight = FontWeight.Medium,
                    letterSpacing = 0.04.em,
                ),
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
    }
}

@Preview
@Composable
private fun LibraryClearanceMeterPreviewLight() {
    AppTheme(darkTheme = false) {
        Surface {
            LibraryClearanceMeter(
                counts =
                    LibraryCounts(
                        total = 66,
                        unread = 42,
                        reading = 5,
                        done = 19,
                        byItemType = mapOf("article" to 38, "video" to 11),
                    ),
            )
        }
    }
}

@Preview
@Composable
private fun LibraryClearanceMeterPreviewDark() {
    AppTheme(darkTheme = true) {
        Surface {
            LibraryClearanceMeter(
                counts =
                    LibraryCounts(
                        total = 12,
                        unread = 12,
                        reading = 0,
                        done = 0,
                        byItemType = mapOf("article" to 12),
                    ),
            )
        }
    }
}
