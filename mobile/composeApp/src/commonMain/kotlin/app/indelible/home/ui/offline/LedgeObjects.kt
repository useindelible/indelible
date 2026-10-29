package app.indelible.home.ui.offline

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.GenericShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.draw.drawWithContent
import androidx.compose.ui.draw.rotate
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.graphics.ColorMatrix
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.graphics.drawscope.DrawScope
import androidx.compose.ui.graphics.lerp
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.layout.layout
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.clearAndSetSemantics
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Constraints
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import app.indelible.home.viewmodel.ShelfItem
import app.indelible.ui.theme.LedgeArt
import app.indelible.ui.theme.MonochromeColors
import app.indelible.ui.theme.OfflineIcons
import coil3.compose.AsyncImage
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.home_offline_cached
import indelible.composeapp.generated.resources.home_offline_kept
import indelible.composeapp.generated.resources.home_offline_not_on_phone
import org.jetbrains.compose.resources.stringResource

internal val STAND_HEIGHT = 144.dp

/**
 * One item the server last listed, standing on the ledge as what it is: a book spine, a stack of PDF sheets,
 * or a clipping for anything else. An item with no copy here is faded and says so before the tap.
 */
@Composable
internal fun LedgeObject(
    item: ShelfItem,
    mono: MonochromeColors,
    onOpen: () -> Unit,
) {
    val away = !item.onPhone
    Column(Modifier.width(OBJECT_WIDTH).clickable(role = Role.Button, onClick = onOpen)) {
        Box(
            contentAlignment = Alignment.BottomCenter,
            modifier =
                Modifier
                    .fillMaxWidth()
                    .height(STAND_HEIGHT)
                    .clearAndSetSemantics {}
                    .drawBehind { drawReflection() },
        ) {
            Box(Modifier.alpha(if (away) AWAY_ALPHA else 1f)) {
                when (item.documentType) {
                    "book" -> BookSpine(item, mono, away)
                    "pdf" -> PdfStack(item, mono)
                    else -> Clipping(item, mono, away)
                }
            }
        }
        Text(
            item.title,
            style = MaterialTheme.typography.bodyMedium.copy(fontWeight = FontWeight.Medium),
            color = if (away) mono.ink2 else mono.ink,
            maxLines = 2,
            overflow = TextOverflow.Ellipsis,
            modifier = Modifier.padding(top = 14.dp),
        )
        Caption(item, mono)
    }
}

@Composable
private fun Caption(
    item: ShelfItem,
    mono: MonochromeColors,
) {
    val kept = item.onPhone && item.pinned
    Row(verticalAlignment = Alignment.CenterVertically, modifier = Modifier.padding(top = 4.dp, start = 1.dp)) {
        if (kept) Icon(OfflineIcons.Pin, null, tint = mono.success, modifier = Modifier.padding(end = 3.dp).size(10.dp))
        val label =
            when {
                !item.onPhone -> Res.string.home_offline_not_on_phone
                kept -> Res.string.home_offline_kept
                else -> Res.string.home_offline_cached
            }
        Text(stringResource(label), style = mono(), color = if (kept) mono.success else mono.ink3, maxLines = 1)
    }
}

@Composable
private fun BookSpine(
    item: ShelfItem,
    mono: MonochromeColors,
    away: Boolean,
) {
    val cloth = LedgeArt.cloths[item.documentId.hashCode().mod(LedgeArt.cloths.size)]
    val ramp = cloth.ramp.map { if (away) it.muted() else it }
    Box(Modifier.size(SPINE_WIDTH, SPINE_HEIGHT)) {
        Box(
            Modifier
                .matchParentSize()
                .shadow(SHADOW, SPINE_SHAPE)
                .background(Brush.horizontalGradient(0f to ramp[0], CLOTH_MID to ramp[1], 1f to ramp[2]), SPINE_SHAPE)
                .drawBehind {
                    val inset = 5.dp.toPx()
                    val band = 11.dp.toPx()
                    drawLine(cloth.band, Offset(inset, band), Offset(size.width - inset, band), 1.dp.toPx())
                    val bottom = size.height - band
                    drawLine(cloth.band, Offset(inset, bottom), Offset(size.width - inset, bottom), 1.dp.toPx())
                },
        )
        Text(
            item.title,
            style = serif(MaterialTheme.typography.labelSmall, FontWeight.Normal),
            color = cloth.ink,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
            modifier = Modifier.align(Alignment.Center).runningUp(SPINE_TITLE),
        )
        item.progressPercent?.let { Ribbon(it / PERCENT, mono.boneMid, Modifier.matchParentSize()) }
    }
}

/** The ribbon hangs out of the spine at the page reached: the top band at 0, the bottom band at 1. */
@Composable
private fun Ribbon(
    read: Float,
    colour: Color,
    modifier: Modifier,
) {
    Canvas(modifier) {
        val w = 19.dp.toPx()
        val h = 7.dp.toPx()
        val x = size.width - 3.dp.toPx()
        val y = 14.dp.toPx() + (size.height - 36.dp.toPx()) * read.coerceIn(0f, 1f)
        val tail =
            Path().apply {
                moveTo(x, y)
                lineTo(x + w, y)
                lineTo(x + w * NOTCH, y + h / 2)
                lineTo(x + w, y + h)
                lineTo(x, y + h)
                close()
            }
        drawPath(tail, colour)
    }
}

@Composable
private fun PdfStack(
    item: ShelfItem,
    mono: MonochromeColors,
) {
    Box(Modifier.size(SHEET_WIDTH, SHEET_HEIGHT)) {
        BACK_SHEETS.forEach { (lift, tilt, fade) ->
            Box(
                Modifier
                    .matchParentSize()
                    .offset(lift, -lift)
                    .rotate(tilt)
                    .alpha(fade)
                    .background(mono.paperBack, RoundedCornerShape(2.dp)),
            )
        }
        Printed(
            mono,
            SHEET_SHAPE,
            SHEET_SHAPE,
            Modifier
                .matchParentSize()
                .drawWithContent {
                    drawContent()
                    val fold = size.width * FOLD
                    val corner =
                        Path().apply {
                            moveTo(size.width - fold, 0f)
                            lineTo(size.width, fold)
                            lineTo(size.width - fold, fold)
                            close()
                        }
                    drawPath(corner, mono.paperBack)
                }.padding(start = 10.dp, end = 9.dp, top = 11.dp, bottom = 8.dp),
        ) {
            Headline(item.title, mono)
            Rule(mono, 1f)
            Rule(mono, RULE_SHORT)
            Spacer(Modifier.weight(1f))
            Footer(kindLabel(item.documentType), mono)
        }
    }
}

@Composable
private fun Clipping(
    item: ShelfItem,
    mono: MonochromeColors,
    away: Boolean,
) {
    Printed(
        mono,
        TORN_SHAPE,
        PAPER_SHADOW,
        Modifier.size(CLIP_WIDTH, CLIP_HEIGHT).padding(start = 8.dp, end = 8.dp, top = 12.dp, bottom = 8.dp),
    ) {
        Picture(item, away)
        Headline(item.title, mono)
        Rule(mono, 1f)
        Rule(mono, RULE_SHORT)
        Spacer(Modifier.weight(1f))
        Footer(item.domain ?: kindLabel(item.documentType), mono)
    }
}

@Composable
private fun Printed(
    mono: MonochromeColors,
    shape: Shape,
    caster: Shape,
    modifier: Modifier,
    content: @Composable ColumnScope.() -> Unit,
) {
    Column(
        verticalArrangement = Arrangement.spacedBy(3.dp),
        modifier = Modifier.shadow(SHADOW, caster).background(mono.paper, shape).then(modifier),
        content = content,
    )
}

@Composable
private fun Picture(
    item: ShelfItem,
    away: Boolean,
) {
    val ramp = LedgeArt.pictures[item.documentId.hashCode().mod(LedgeArt.pictures.size)]
    Box(
        Modifier
            .fillMaxWidth()
            .height(34.dp)
            .clip(RoundedCornerShape(1.dp))
            .background(Brush.linearGradient(ramp.map { if (away) it.muted() else it })),
    ) {
        if (item.imageUrl != null) {
            AsyncImage(
                model = item.imageUrl,
                contentDescription = null,
                contentScale = ContentScale.Crop,
                colorFilter = if (away) MUTED_FILTER else null,
                modifier = Modifier.matchParentSize(),
            )
        }
    }
}

@Composable
private fun Headline(
    title: String,
    mono: MonochromeColors,
) {
    Text(
        title,
        style = serif(MaterialTheme.typography.labelSmall, FontWeight.Medium),
        color = mono.paperInk,
        maxLines = 2,
        overflow = TextOverflow.Ellipsis,
    )
}

@Composable
private fun Rule(
    mono: MonochromeColors,
    length: Float,
) {
    Box(Modifier.fillMaxWidth(length).height(2.dp).background(mono.paperRule, RoundedCornerShape(1.dp)))
}

@Composable
private fun Footer(
    text: String,
    mono: MonochromeColors,
) {
    Text(
        text,
        style = mono(),
        color = mono.paperInk.copy(alpha = FOOTER_ALPHA),
        maxLines = 1,
        overflow = TextOverflow.Ellipsis,
    )
}

private fun DrawScope.drawReflection() {
    val inset = 14.dp.toPx()
    val top = size.height + 1.dp.toPx()
    val oval = Size(size.width - 2 * inset, 9.dp.toPx())
    val centre = Offset(size.width / 2, top + oval.height / 2)
    drawOval(
        Brush.radialGradient(listOf(LedgeArt.reflection, Color.Transparent), centre, oval.width / 2),
        Offset(inset, top),
        oval,
    )
}

/** Lays the text out [length] long and turns it to run up the spine, bottom to top. */
private fun Modifier.runningUp(length: Dp): Modifier =
    layout { measurable, _ ->
        val text = measurable.measure(Constraints(maxWidth = length.roundToPx()))
        layout(text.height, text.width) {
            text.placeWithLayer((text.height - text.width) / 2, (text.width - text.height) / 2) {
                rotationZ = UP
            }
        }
    }

private fun Color.muted(): Color {
    val grey = red * LUMA_R + green * LUMA_G + blue * LUMA_B
    return lerp(this, Color(grey, grey, grey, alpha), 1f - AWAY_SATURATION)
}

private val MUTED_FILTER = ColorFilter.colorMatrix(ColorMatrix().apply { setToSaturation(AWAY_SATURATION) })

// Android before 10 casts no shadow from a concave outline, so the torn clipping casts its box's.
private val PAPER_SHADOW = RoundedCornerShape(2.dp)

private val SPINE_SHAPE = RoundedCornerShape(topStart = 2.dp, bottomStart = 2.dp, topEnd = 5.dp, bottomEnd = 5.dp)

private val SHEET_SHAPE =
    GenericShape { size, _ ->
        moveTo(0f, 0f)
        lineTo(size.width * (1 - FOLD), 0f)
        lineTo(size.width, size.width * FOLD)
        lineTo(size.width, size.height)
        lineTo(0f, size.height)
        close()
    }

// Stepping by a number coprime with the depth count gives a ragged edge that never repeats a pattern you can see.
private val TORN_SHAPE =
    GenericShape { size, _ ->
        val step = size.height / CLIP_HEIGHT.value
        for (tooth in 0..TEETH) {
            val x = size.width * tooth / TEETH
            val y = (TEAR_TOP + (tooth * TEAR_STRIDE) % TEAR_DEPTHS) * step
            if (tooth == 0) moveTo(x, y) else lineTo(x, y)
        }
        lineTo(size.width, size.height)
        lineTo(0f, size.height)
        close()
    }

private val OBJECT_WIDTH = 98.dp
private val SPINE_WIDTH = 38.dp
private val SPINE_HEIGHT = 134.dp
private val SPINE_TITLE = 100.dp
private val SHEET_WIDTH = 76.dp
private val SHEET_HEIGHT = 104.dp
private val CLIP_WIDTH = 80.dp
private val CLIP_HEIGHT = 118.dp
private val SHADOW = 10.dp
private const val AWAY_ALPHA = 0.45f
private const val AWAY_SATURATION = 0.25f
private const val CLOTH_MID = 0.6f
private const val NOTCH = 0.84f
private const val FOLD = 0.2f
private const val RULE_SHORT = 0.72f
private const val FOOTER_ALPHA = 0.6f
private const val BACK_TILT = 1.5f
private const val BACK_ALPHA = 0.75f
private const val MIDDLE_TILT = 0.6f
private const val MIDDLE_ALPHA = 0.9f
private val BACK_SHEETS = listOf(Triple(6.dp, BACK_TILT, BACK_ALPHA), Triple(3.dp, MIDDLE_TILT, MIDDLE_ALPHA))
private const val TEETH = 12
private const val TEAR_TOP = 1
private const val TEAR_STRIDE = 5
private const val TEAR_DEPTHS = 8
private const val PERCENT = 100f
private const val UP = -90f
private const val LUMA_R = 0.299f
private const val LUMA_G = 0.587f
private const val LUMA_B = 0.114f
