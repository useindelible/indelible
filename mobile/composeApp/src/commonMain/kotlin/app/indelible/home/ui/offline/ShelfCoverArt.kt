package app.indelible.home.ui.offline

import androidx.compose.foundation.shape.GenericShape
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.drawscope.ContentDrawScope
import androidx.compose.ui.graphics.drawscope.DrawScope
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.unit.dp
import app.indelible.ui.theme.CoverArt

/** The generated covers: every literal is a proportion of the cover or a size in dp. */
internal val RIBBON =
    GenericShape { size, _ ->
        moveTo(0f, 0f)
        lineTo(size.width, 0f)
        lineTo(size.width, size.height)
        lineTo(size.width / 2, size.height * 0.72f)
        lineTo(0f, size.height)
        close()
    }

internal fun DrawScope.drawArt(style: CoverStyle) {
    when (style) {
        CoverStyle.EMBER -> {
            drawRect(Brush.linearGradient(CoverArt.ember.first, Offset(size.width, 0f), Offset(0f, size.height)))
            val sun = Offset(size.width * 0.72f, size.height * 0.34f)
            val reach = farthestCorner(sun)
            drawCircle(CoverArt.ember.second.copy(alpha = 0.35f), reach * 0.265f, sun, style = Stroke(reach * 0.01f))
            drawCircle(CoverArt.ember.second, reach * 0.17f, sun)
        }
        CoverStyle.OXBLOOD -> {
            drawRect(Brush.linearGradient(CoverArt.oxblood.first, Offset(size.width, 0f), Offset(0f, size.height)))
            val origin = Offset(size.width / 2, size.height * 1.18f)
            var r = 9.dp.toPx()
            while (r < size.height * 1.2f) {
                drawCircle(CoverArt.oxblood.second, r, origin, style = Stroke(1.2.dp.toPx()))
                r += 9.dp.toPx()
            }
        }
        CoverStyle.COAST -> {
            val horizon = size.height * 0.47f
            drawRect(Brush.verticalGradient(CoverArt.coastSky, 0f, horizon), size = size.copy(height = horizon))
            val sun = Offset(size.width / 2, horizon)
            drawCircle(CoverArt.coastSun, farthestCorner(sun) * 0.11f, sun)
            drawRect(
                Brush.verticalGradient(CoverArt.coastSea, horizon, size.height),
                Offset(0f, horizon),
                size.copy(height = size.height - horizon),
            )
        }
        CoverStyle.BONE -> {
            drawRect(CoverArt.bone.first)
            val corner = Offset(size.width, size.height)
            drawCircle(CoverArt.bone.second, farthestCorner(corner) * 0.3f, corner)
        }
        CoverStyle.TIDE -> {
            drawRect(Brush.linearGradient(CoverArt.tide.first, Offset(size.width, 0f), Offset(0f, size.height)))
            val wave = Size(WAVE_RX.dp.toPx() * 2, WAVE_RY.dp.toPx() * 2)
            var y = TILE_H.dp.toPx()
            while (y < size.height) {
                var x = 0f
                while (x < size.width) {
                    val crest = Offset(x + TILE_W.dp.toPx() / 2 - wave.width / 2, y - wave.height / 2)
                    drawArc(CoverArt.tide.second, 180f, 180f, false, crest, wave, style = Stroke(1.dp.toPx()))
                    x += TILE_W.dp.toPx()
                }
                y += TILE_H.dp.toPx()
            }
        }
    }
}

internal fun ContentDrawScope.drawSpineAndGloss() {
    drawContent()
    val spine = 7.dp.toPx()
    drawRect(
        Brush.horizontalGradient(
            0f to Color.Black.copy(alpha = 0.4f),
            0.55f to Color.White.copy(alpha = 0.14f),
            1f to Color.Transparent,
            endX = spine,
        ),
        size = size.copy(width = spine),
    )
    drawRect(
        Brush.linearGradient(
            0f to Color.White.copy(alpha = 0.2f),
            0.36f to Color.Transparent,
            start = Offset.Zero,
            end = Offset(size.width, size.height * 0.7f),
        ),
    )
}

/** Fine horizontal lines on the right edge: the pages behind the board. */
internal fun DrawScope.drawPageEdges() {
    val gap = 2.dp.toPx()
    var y = gap
    while (y < size.height) {
        val inset = size.width - PAGE_DEPTH.toPx() * 1.6f
        drawLine(Color.Black.copy(alpha = 0.12f), Offset(inset, y), Offset(size.width, y), 0.6f)
        y += gap
    }
}

/** Circle sizes are fractions of the distance from their centre to the farthest corner. */
private fun DrawScope.farthestCorner(center: Offset): Float =
    listOf(Offset.Zero, Offset(size.width, 0f), Offset(0f, size.height), Offset(size.width, size.height))
        .maxOf { (it - center).getDistance() }

internal enum class CoverStyle { EMBER, OXBLOOD, COAST, BONE, TIDE }

internal val PAGE_DEPTH = 3.dp
private const val TILE_W = 16f
private const val TILE_H = 10f
private const val WAVE_RX = 9.4f
private const val WAVE_RY = 4.7f
