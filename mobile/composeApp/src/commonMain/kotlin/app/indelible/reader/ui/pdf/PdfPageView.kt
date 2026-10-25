package app.indelible.reader.ui.pdf

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.gestures.detectDragGesturesAfterLongPress
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.produceState
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.CornerRadius
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.FilterQuality
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.graphics.drawscope.DrawScope
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.unit.IntSize
import app.indelible.reader.pdf.NormalizedPoint
import app.indelible.reader.pdf.NormalizedRect
import app.indelible.reader.pdf.PdfHighlightBox
import app.indelible.reader.pdf.PdfPageSize
import app.indelible.reader.pdf.PdfTone
import app.indelible.reader.pdf.renderWidthFor
import app.indelible.reader.pdf.toneBackground
import app.indelible.ui.theme.HIGHLIGHT_FILL_ALPHA
import app.indelible.ui.theme.HighlightFills
import app.indelible.ui.theme.IndelibleShape
import app.indelible.ui.theme.forApiColor
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.reader_pdf_page
import kotlinx.coroutines.CancellationException
import org.jetbrains.compose.resources.stringResource
import kotlin.math.roundToInt

/** Drag points arrive as fractions of the page, clamped inside it. */
internal class PdfPageGestures(
    val onTap: (NormalizedPoint) -> Unit,
    val onDragStart: (NormalizedPoint) -> Unit,
    val onDrag: (NormalizedPoint) -> Unit,
    val onDragEnd: () -> Unit,
    val onDragCancel: () -> Unit,
)

internal class PdfPageOverlay(
    val highlights: List<PdfHighlightBox>,
    val fills: HighlightFills,
    val selection: List<NormalizedRect>,
    val selectionColor: Color,
)

@Composable
internal fun PdfPageView(
    index: Int,
    pageCount: Int,
    page: PdfPageSize,
    layoutWidthPx: Int,
    bitmaps: PdfPageBitmaps,
    tone: PdfTone,
    overlay: PdfPageOverlay,
    gestures: PdfPageGestures,
    modifier: Modifier = Modifier,
) {
    val label = stringResource(Res.string.reader_pdf_page, index + 1, pageCount)
    val renderWidth = renderWidthFor(page, layoutWidthPx)
    val bitmap by produceState<ImageBitmap?>(null, bitmaps, index, renderWidth, tone) {
        value = bitmaps.tryBitmap(index, renderWidth, tone)
    }
    val current = rememberUpdatedState(gestures)
    val placeholder = remember(tone) { toneBackground(tone) }
    Canvas(
        modifier
            .semantics { contentDescription = label }
            .pointerInput(Unit) {
                detectTapGestures { current.value.onTap(normalized(it, size)) }
            }.pointerInput(Unit) {
                detectDragGesturesAfterLongPress(
                    onDragStart = { current.value.onDragStart(normalized(it, size)) },
                    onDragEnd = { current.value.onDragEnd() },
                    onDragCancel = { current.value.onDragCancel() },
                    onDrag = { change, _ -> current.value.onDrag(normalized(change.position, size)) },
                )
            },
    ) {
        drawRect(placeholder)
        bitmap?.let {
            val dst = IntSize(size.width.roundToInt(), size.height.roundToInt())
            drawImage(it, dstSize = dst, filterQuality = FilterQuality.Medium)
        }
        val radius = CornerRadius(IndelibleShape.highlight.topStart.toPx(size, this))
        overlay.highlights.forEach { box ->
            val fill = overlay.fills.forApiColor(box.color) ?: return@forEach
            box.rects.forEach { drawBox(it, fill, radius) }
        }
        overlay.selection.forEach { drawBox(it, overlay.selectionColor, radius) }
    }
}

private fun DrawScope.drawBox(
    rect: NormalizedRect,
    color: Color,
    radius: CornerRadius,
) {
    val topLeft = Offset((rect.x * size.width).toFloat(), (rect.y * size.height).toFloat())
    val boxSize = Size((rect.width * size.width).toFloat(), (rect.height * size.height).toFloat())
    drawRoundRect(color, topLeft, boxSize, radius, alpha = HIGHLIGHT_FILL_ALPHA)
}

private fun normalized(
    position: Offset,
    size: IntSize,
) = NormalizedPoint(
    (position.x / size.width.coerceAtLeast(1)).toDouble().coerceIn(0.0, 1.0),
    (position.y / size.height.coerceAtLeast(1)).toDouble().coerceIn(0.0, 1.0),
)

// A page whose render fails (the copy went away, the engine refused) stays on its placeholder.
private suspend fun PdfPageBitmaps.tryBitmap(
    index: Int,
    widthPx: Int,
    tone: PdfTone,
): ImageBitmap? =
    runCatching { bitmap(index, widthPx, tone) }.getOrElse { error ->
        if (error is CancellationException) throw error
        null
    }
