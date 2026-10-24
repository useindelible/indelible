package app.indelible.offline.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.selection.selectable
import androidx.compose.foundation.selection.selectableGroup
import androidx.compose.foundation.shape.CornerSize
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.RectangleShape
import androidx.compose.ui.graphics.drawOutline
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.graphics.drawscope.translate
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.unit.TextUnit
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.em
import app.indelible.ui.theme.IndelibleShape
import app.indelible.ui.theme.IndelibleSpacing
import app.indelible.ui.theme.IndelibleTheme
import app.indelible.ui.theme.geistMonoFontFamily

internal val HAIRLINE = 1.dp
internal val SECTION_SPACING = IndelibleSpacing.step40
private val SEGMENT_HEIGHT = 32.dp
private val ACTION_HEIGHT = 28.dp
private const val COUNT_ALPHA = 0.72f
private const val THUMB_AMBIENT_ALPHA = 0.08f
private const val THUMB_SPOT_ALPHA = 0.04f
internal val TRACK_LABEL = 0.16.em
internal val TRACK_META = 0.05.em
private val TRACK_SEGMENT = 0.04.em
private val TRACK_ACTION = 0.1.em
private val CARD_SHAPE = IndelibleShape.xl
private val ZERO_CORNER = CornerSize(0.dp)
private const val ACCENT_WASH = 0.10f
private const val ACCENT_EDGE = 0.28f

@Composable
internal fun storageMono(
    tracking: TextUnit,
    weight: FontWeight = FontWeight.SemiBold,
): TextStyle =
    MaterialTheme.typography.labelSmall.copy(
        fontFamily = geistMonoFontFamily(),
        fontWeight = weight,
        letterSpacing = tracking,
    )

@Composable
internal fun SectionLabel(text: String) {
    Text(
        text = text,
        style = storageMono(TRACK_LABEL),
        color = IndelibleTheme.colors.textTertiary,
    )
}

/** A section label with its count on the right, sitting on one baseline. */
@Composable
internal fun SectionHead(
    label: String,
    count: String,
    modifier: Modifier = Modifier,
) {
    Row(modifier = modifier.fillMaxWidth()) {
        Box(modifier = Modifier.weight(1f).alignByBaseline()) { SectionLabel(label) }
        Text(
            text = count,
            style = storageMono(TRACK_META, FontWeight.Normal),
            color = IndelibleTheme.colors.textTertiary,
            modifier = Modifier.alignByBaseline(),
        )
    }
}

internal class Segment(
    val label: String,
    val count: Int? = null,
)

/** A track with one raised thumb, the switcher both the cap and the download filter use. */
@Composable
internal fun StorageSegmented(
    segments: List<Segment>,
    selected: Int,
    onSelect: (Int) -> Unit,
    modifier: Modifier = Modifier,
) {
    val colors = MaterialTheme.colorScheme
    Row(
        modifier =
            modifier
                .fillMaxWidth()
                .clip(IndelibleShape.md)
                .background(colors.surfaceVariant)
                .border(HAIRLINE, colors.outlineVariant, IndelibleShape.md)
                .padding(IndelibleSpacing.step2)
                .selectableGroup(),
        horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step2),
    ) {
        segments.forEachIndexed { index, segment ->
            val isSelected = index == selected
            val ink = if (isSelected) colors.onSurface else IndelibleTheme.colors.textTertiary
            Box(
                modifier =
                    Modifier
                        .weight(1f)
                        .height(SEGMENT_HEIGHT)
                        .then(if (isSelected) Modifier.thumb() else Modifier)
                        .clip(IndelibleShape.sm)
                        .selectable(selected = isSelected, role = Role.Tab, onClick = { onSelect(index) }),
                contentAlignment = Alignment.Center,
            ) {
                Text(
                    text = segment.text(ink),
                    style = storageMono(TRACK_SEGMENT),
                    color = ink,
                    maxLines = 1,
                )
            }
        }
    }
}

private fun Segment.text(ink: Color) =
    buildAnnotatedString {
        append(label)
        count?.let {
            append(" ")
            withStyle(SpanStyle(fontWeight = FontWeight.Medium, color = ink.copy(alpha = COUNT_ALPHA))) {
                append(it.toString())
            }
        }
    }

@Composable
private fun Modifier.thumb(): Modifier {
    val colors = MaterialTheme.colorScheme
    return shadow(
        elevation = IndelibleSpacing.step2,
        shape = IndelibleShape.sm,
        ambientColor = colors.onSurface.copy(alpha = THUMB_AMBIENT_ALPHA),
        spotColor = colors.onSurface.copy(alpha = THUMB_SPOT_ALPHA),
    ).background(colors.surfaceContainer, IndelibleShape.sm)
        .border(HAIRLINE, colors.outlineVariant, IndelibleShape.sm)
}

/**
 * One row of a bordered card that a lazy list draws row by row: [first] and [last] round the card's
 * ends, and every row after the first draws the divider above it.
 */
internal fun Modifier.cardCell(
    first: Boolean,
    last: Boolean,
    fill: Color,
    edge: Color,
): Modifier =
    clip(cellShape(first, last))
        .background(fill)
        .drawBehind {
            val line = HAIRLINE.toPx()
            val reach = size.height
            val top = if (first) 0f else -reach
            val bottom = if (last) size.height else size.height + reach
            val outline = CARD_SHAPE.createOutline(Size(size.width, bottom - top), layoutDirection, this)
            translate(top = top) { drawOutline(outline, edge, style = Stroke(line * 2)) }
            if (!first) drawLine(edge, Offset(0f, line / 2), Offset(size.width, line / 2), line)
        }

private fun cellShape(
    first: Boolean,
    last: Boolean,
) = when {
    first && last -> CARD_SHAPE
    first -> CARD_SHAPE.copy(bottomStart = ZERO_CORNER, bottomEnd = ZERO_CORNER)
    last -> CARD_SHAPE.copy(topStart = ZERO_CORNER, topEnd = ZERO_CORNER)
    else -> RectangleShape
}

/** The small raised action a storage row carries; [accent] marks the one that resolves a problem. */
@Composable
internal fun RowAction(
    text: String,
    onClick: () -> Unit,
    accent: Boolean = false,
) {
    val colors = MaterialTheme.colorScheme
    val ink = if (accent) colors.primary else colors.onSurfaceVariant
    val edge = if (accent) colors.primary.copy(alpha = ACCENT_EDGE) else colors.outlineVariant
    Box(
        modifier =
            Modifier
                .height(ACTION_HEIGHT)
                .clip(IndelibleShape.sm)
                .background(if (accent) colors.primary.copy(alpha = ACCENT_WASH) else colors.surfaceContainer)
                .border(HAIRLINE, edge, IndelibleShape.sm)
                .clickable(role = Role.Button, onClick = onClick)
                .padding(horizontal = IndelibleSpacing.step10),
        contentAlignment = Alignment.Center,
    ) {
        Text(text = text, style = storageMono(TRACK_ACTION), color = ink, maxLines = 1)
    }
}
