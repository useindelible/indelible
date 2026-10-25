package app.indelible.reader.ui.pdf

import androidx.compose.foundation.ScrollState
import androidx.compose.foundation.gestures.scrollBy
import androidx.compose.foundation.lazy.LazyListLayoutInfo
import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.runtime.snapshotFlow
import androidx.compose.runtime.withFrameNanos
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.semantics.CustomAccessibilityAction
import androidx.compose.ui.unit.Density
import app.indelible.reader.pdf.NormalizedPoint
import app.indelible.reader.pdf.NormalizedRect
import app.indelible.reader.pdf.PdfDocument
import app.indelible.reader.pdf.PdfHighlightBox
import app.indelible.reader.pdf.PdfSelection
import app.indelible.reader.ui.SelectionRect
import app.indelible.reader.viewmodel.PdfScrollSample
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.launch
import kotlin.math.roundToInt

internal class PdfViewCallbacks(
    val onSelection: (PdfSelection?, SelectionRect?) -> Unit,
    val onHighlightTapped: (highlightId: String) -> Unit,
    val onReaderTap: () -> Unit,
)

/** Everything the page stack needs from the reader view: the open document, its caches, and the gesture handlers. */
internal class PdfPagesHost(
    val document: PdfDocument,
    val bitmaps: PdfPageBitmaps,
    val selection: PdfSelectionState,
    val layout: PdfLayout,
    val zoom: PdfZoomState,
    private val scope: CoroutineScope,
) {
    fun commitPinch(
        scale: Float,
        centroid: Offset,
    ) {
        val list = layout.listState
        val first = LazyAnchor(list.firstVisibleItemIndex, list.firstVisibleItemScrollOffset)
        val focus = centroid - Offset(0f, layout.topInsetPx.toFloat())
        val commit = commitZoom(zoom.zoom, scale, focus, first, layout.horizontal.value)
        val widthBefore = list.layoutInfo.viewportSize.width
        val target = (layout.viewportWidthPx * commit.zoom).roundToInt().coerceAtLeast(1)
        zoom.zoom = commit.zoom
        zoom.preview = null
        list.requestScrollToItem(commit.anchor.index, commit.anchor.offsetPx.coerceAtLeast(0))
        scope.launch {
            // The horizontal range only exists once the list has measured at its new width.
            if (target != widthBefore) {
                snapshotFlow { list.layoutInfo.viewportSize.width }.first { it == target }
                withFrameNanos { }
            }
            if (commit.anchor.offsetPx < 0) list.scrollBy(commit.anchor.offsetPx.toFloat())
            layout.horizontal.scrollTo(commit.horizontalScroll)
        }
    }

    fun gestures(
        index: Int,
        boxes: List<PdfHighlightBox>,
        callbacks: PdfViewCallbacks,
    ) = PdfPageGestures(
        onTap = { point -> tap(point, boxes, callbacks) },
        onDragStart = { selection.start(index, it) },
        onDrag = selection::move,
        onDragEnd = {
            selection.end { result ->
                callbacks.onSelection(result, result?.let { layout.anchorRect(index, it) })
            }
        },
        onDragCancel = selection::cancel,
    )

    /** One action per highlight, each named by [labels] at the same index, so a screen reader can tell them apart. */
    fun semantics(
        boxes: List<PdfHighlightBox>,
        labels: List<String>,
        callbacks: PdfViewCallbacks,
    ) = PdfPageSemantics(
        onActivate = callbacks.onReaderTap,
        highlightActions =
            boxes.zip(labels) { box, label ->
                CustomAccessibilityAction(label) {
                    callbacks.onHighlightTapped(box.highlightId)
                    true
                }
            },
    )

    private fun tap(
        point: NormalizedPoint,
        boxes: List<PdfHighlightBox>,
        callbacks: PdfViewCallbacks,
    ) {
        val hit = boxes.firstOrNull { box -> box.rects.any { it.contains(point) } }
        if (hit != null) {
            callbacks.onHighlightTapped(hit.highlightId)
            return
        }
        if (selection.clear()) callbacks.onSelection(null, null)
        callbacks.onReaderTap()
    }
}

private fun NormalizedRect.contains(point: NormalizedPoint) = point.x in x..(x + width) && point.y in y..(y + height)

/** Where the laid-out pages are, for turning list frames into samples and selections into toolbar anchors. */
internal data class PdfPageGrid(
    val viewportWidthPx: Int,
    val itemWidthPx: Int,
    val heightsPx: List<Int>,
    val gapPx: Int,
)

internal class PdfLayout(
    val listState: LazyListState,
    val horizontal: ScrollState,
    private val grid: PdfPageGrid,
    val topInsetPx: Int,
    private val density: Density,
) {
    val viewportWidthPx: Int get() = grid.viewportWidthPx
    val itemWidthPx: Int get() = grid.itemWidthPx
    val heightsPx: List<Int> get() = grid.heightsPx

    fun sample(
        info: LazyListLayoutInfo,
        pageCount: Int,
    ): PdfScrollSample? {
        val item = info.visibleItemsInfo.firstOrNull { it.offset + it.size > 0 } ?: return null
        val top = PdfTopItem(item.index, (-item.offset).coerceAtLeast(0), item.size)
        val last = info.visibleItemsInfo.lastOrNull()
        val atEnd = last != null && last.index == pageCount - 1 && last.offset + last.size <= info.viewportEndOffset
        val scrollTopPx = (0 until top.index).sumOf { heightsPx[it] + grid.gapPx } + top.offsetPx
        return sampleOf(top, pageCount, atEnd, scrollTopPx / density.density)
    }

    fun anchorRect(
        page: Int,
        selection: PdfSelection,
    ): SelectionRect? {
        val rect = selection.rects.firstOrNull() ?: return null
        val item = listState.layoutInfo.visibleItemsInfo.firstOrNull { it.index == page }
        return item?.let {
            val topPx = it.offset + topInsetPx
            val heightPx = heightsPx[page]
            SelectionRect(
                x = ((rect.x * itemWidthPx - horizontal.value) / density.density).toFloat(),
                y = ((topPx + rect.y * heightPx) / density.density).toFloat(),
                width = (rect.width * itemWidthPx / density.density).toFloat(),
                height = (rect.height * heightPx / density.density).toFloat(),
            )
        }
    }
}
