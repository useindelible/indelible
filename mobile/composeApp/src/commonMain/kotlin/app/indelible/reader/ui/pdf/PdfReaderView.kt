package app.indelible.reader.ui.pdf

import androidx.compose.foundation.background
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.rememberScrollState
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.runtime.snapshotFlow
import androidx.compose.runtime.withFrameNanos
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clipToBounds
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.TransformOrigin
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.platform.LocalDensity
import app.indelible.reader.pdf.PdfDocument
import app.indelible.reader.pdf.PdfHighlightBox
import app.indelible.reader.pdf.PdfSelection
import app.indelible.reader.pdf.PdfTone
import app.indelible.reader.ui.SelectionRect
import app.indelible.reader.viewmodel.PdfRestoreRequest
import app.indelible.reader.viewmodel.PdfScrollSample
import app.indelible.ui.theme.HighlightFills
import app.indelible.ui.theme.IndelibleSpacing
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.reader_pdf_open_highlight
import kotlinx.coroutines.flow.first
import org.jetbrains.compose.resources.stringResource
import kotlin.math.roundToInt

private const val NO_NONCE = -1L

/** The pages of one open PDF, hidden until the saved place is on screen; [onSelection] gets a toolbar anchor in dp. */
@Composable
internal fun PdfReaderView(
    document: PdfDocument,
    tone: PdfTone,
    gutter: Color,
    highlightsByPage: Map<Int, List<PdfHighlightBox>>,
    highlightFills: HighlightFills,
    restore: PdfRestoreRequest?,
    activeSelection: PdfSelection?,
    topInsetPx: Int,
    onPositioned: () -> Unit,
    onSample: (PdfScrollSample) -> Unit,
    onSelection: (PdfSelection?, SelectionRect?) -> Unit,
    onHighlightTapped: (highlightId: String) -> Unit,
    onReaderTap: () -> Unit,
    modifier: Modifier = Modifier,
    zoom: PdfZoomState = rememberPdfZoomState(),
) {
    val density = LocalDensity.current
    val scope = rememberCoroutineScope()
    val listState = rememberLazyListState()
    val horizontal = rememberScrollState()
    val bitmaps = remember(document) { PdfPageBitmaps(document) }
    val selection = remember(document) { PdfSelectionState(document, scope) }
    var revealed by rememberSaveable(document) { mutableStateOf(false) }
    var applied by rememberSaveable(document) { mutableStateOf(NO_NONCE) }
    LaunchedEffect(selection) { selection.followDrags() }
    LaunchedEffect(activeSelection) { if (activeSelection == null) selection.clear() }

    BoxWithConstraints(modifier.background(gutter).clipToBounds()) {
        val itemWidthPx = (constraints.maxWidth * zoom.zoom).roundToInt().coerceAtLeast(1)
        val heightsPx = remember(document, itemWidthPx) { document.pageSizes.map { pageHeightPx(it, itemWidthPx) } }
        val gapPx = with(density) { IndelibleSpacing.step8.roundToPx() }
        val grid = PdfPageGrid(constraints.maxWidth, itemWidthPx, heightsPx, gapPx)
        val layout = PdfLayout(listState, horizontal, grid, topInsetPx, density)
        val host = PdfPagesHost(document, bitmaps, selection, layout, zoom, scope)
        val callbacks = PdfViewCallbacks(onSelection, onHighlightTapped, onReaderTap)

        // A recreated view keeps the list where the reader left it; only a new restore request scrolls.
        LaunchedEffect(restore, document) {
            val target = restore ?: return@LaunchedEffect
            if (heightsPx.isEmpty()) return@LaunchedEffect
            if (applied != target.nonce) {
                val index = target.pageIndex.coerceIn(0, heightsPx.lastIndex)
                listState.scrollToItem(index, restoreOffsetPx(target.pageOffset, heightsPx[index]))
                applied = target.nonce
            }
            revealed = true
            onPositioned()
        }
        val currentLayout by rememberUpdatedState(layout)
        val currentOnSample by rememberUpdatedState(onSample)
        var latestPage by rememberSaveable(document) { mutableStateOf(0) }
        var latestOffset by rememberSaveable(document) { mutableStateOf(0f) }
        var viewportSeen by rememberSaveable(document) { mutableStateOf(constraints.maxWidth) }
        // A rotation changes the page heights under the list; the place in the page is kept, not the px offset.
        LaunchedEffect(constraints.maxWidth) {
            if (viewportSeen == constraints.maxWidth) return@LaunchedEffect
            viewportSeen = constraints.maxWidth
            if (latestPage == 0 || heightsPx.isEmpty()) return@LaunchedEffect
            val index = (latestPage - 1).coerceIn(0, heightsPx.lastIndex)
            val pageOffset = latestOffset
            snapshotFlow { listState.layoutInfo.viewportSize.width }.first { it == itemWidthPx }
            withFrameNanos {}
            listState.scrollToItem(index, restoreOffsetPx(pageOffset, heightsPx[index]))
        }
        LaunchedEffect(listState, document) {
            snapshotFlow { listState.layoutInfo }.collect { info ->
                if (revealed) {
                    currentLayout.sample(info, document.pageCount)?.let { sample ->
                        latestPage = sample.page
                        latestOffset = sample.pageOffset
                        currentOnSample(sample)
                    }
                }
            }
        }

        PdfPageStack(host, revealed, tone, highlightsByPage, highlightFills, callbacks)
    }
}

@Composable
private fun PdfPageStack(
    host: PdfPagesHost,
    shown: Boolean,
    tone: PdfTone,
    highlightsByPage: Map<Int, List<PdfHighlightBox>>,
    fills: HighlightFills,
    callbacks: PdfViewCallbacks,
) {
    val density = LocalDensity.current
    val selectionColor = MaterialTheme.colorScheme.primary
    val openHighlightLabel = stringResource(Res.string.reader_pdf_open_highlight)
    val current by rememberUpdatedState(host)
    Box(
        Modifier
            .fillMaxSize()
            .alpha(if (shown) 1f else 0f)
            .pinchToZoom(host.zoom) { scale, centroid -> current.commitPinch(scale, centroid) }
            .graphicsLayer {
                val preview = host.zoom.preview
                scaleX = preview?.scale ?: 1f
                scaleY = preview?.scale ?: 1f
                transformOrigin =
                    preview?.let { TransformOrigin(it.centroid.x / size.width, it.centroid.y / size.height) }
                        ?: TransformOrigin.Center
            }.horizontalScroll(host.layout.horizontal),
    ) {
        LazyColumn(
            state = host.layout.listState,
            modifier = Modifier.width(with(density) { host.layout.itemWidthPx.toDp() }).fillMaxHeight(),
            contentPadding = PaddingValues(top = with(density) { host.layout.topInsetPx.toDp() }),
            verticalArrangement = Arrangement.spacedBy(IndelibleSpacing.step8),
        ) {
            items(host.document.pageCount, key = { it }) { index ->
                val boxes = highlightsByPage[index].orEmpty()
                val layout = host.layout
                val widthDp = with(density) { layout.itemWidthPx.toDp() }
                val heightDp = with(density) { layout.heightsPx[index].toDp() }
                PdfPageView(
                    index = index,
                    pageCount = host.document.pageCount,
                    page = host.document.pageSizes[index],
                    layoutWidthPx = host.layout.itemWidthPx,
                    bitmaps = host.bitmaps,
                    tone = tone,
                    overlay = PdfPageOverlay(boxes, fills, host.selection.rectsOn(index), selectionColor),
                    gestures = host.gestures(index, boxes, callbacks, openHighlightLabel),
                    modifier = Modifier.size(widthDp, heightDp),
                )
            }
        }
    }
}
