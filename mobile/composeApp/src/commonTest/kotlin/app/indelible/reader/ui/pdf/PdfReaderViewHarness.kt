package app.indelible.reader.ui.pdf

import androidx.compose.foundation.layout.size
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.LocalSaveableStateRegistry
import androidx.compose.runtime.saveable.SaveableStateRegistry
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.test.ComposeUiTest
import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.SemanticsNodeInteraction
import androidx.compose.ui.test.TouchInjectionScope
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import app.indelible.reader.pdf.FakePdfDocument
import app.indelible.reader.pdf.NormalizedPoint
import app.indelible.reader.pdf.NormalizedRect
import app.indelible.reader.pdf.PdfHighlightBox
import app.indelible.reader.pdf.PdfPageSize
import app.indelible.reader.pdf.PdfSelection
import app.indelible.reader.pdf.PdfTone
import app.indelible.reader.ui.SelectionRect
import app.indelible.reader.viewmodel.PdfRestoreRequest
import app.indelible.reader.viewmodel.PdfScrollSample
import app.indelible.ui.theme.AppTheme
import app.indelible.ui.theme.IndelibleTheme
import kotlinx.coroutines.CompletableDeferred
import kotlin.math.roundToInt

internal const val VIEW_TAG = "pdfView"
private const val LONG_PRESS_MS = 1_000L

internal class Reports {
    var density = 1f
    var positioned = 0
    var readerTaps = 0
    val samples = mutableListOf<PdfScrollSample>()
    val selections = mutableListOf<Pair<PdfSelection?, SelectionRect?>>()
    val tapped = mutableListOf<String>()
}

/** Three Letter pages whose engine answers every drag with the drag's end point in the text. */
internal class Scripted(
    val gate: CompletableDeferred<Unit>? = null,
    val gateFrom: Double = 2.0,
) {
    val ends = mutableListOf<NormalizedPoint>()
    val document =
        FakePdfDocument(List(3) { PdfPageSize(612.0, 792.0) }, selectionFor = { page, _, end ->
            ends += end
            if (end.x >= gateFrom) gate?.await()
            val rects = listOf(NormalizedRect(0.2, end.y, 0.3, 0.02))
            PdfSelection(page, "words to ${(end.x * 100).roundToInt()}", 0, 5, rects)
        })
}

/** What a recreated view gets back: the registry of each generation and the state the last one saved. */
internal class SavedViews {
    var state: Map<String, List<Any?>>? = null
    var hidden by mutableStateOf(false)
    val registries = mutableListOf<SaveableStateRegistry>()
}

@OptIn(ExperimentalTestApi::class)
internal fun ComposeUiTest.show(
    document: FakePdfDocument,
    restore: () -> PdfRestoreRequest? = { PdfRestoreRequest(0, 0f, 1) },
    highlights: Map<Int, List<PdfHighlightBox>> = emptyMap(),
    height: Dp = 700.dp,
    zoom: PdfZoomState = PdfZoomState(),
    activeSelection: () -> PdfSelection? = { null },
    saved: SavedViews = SavedViews(),
    generation: () -> Int = { 0 },
): Reports {
    val reports = Reports()
    setContent {
        val registry = remember(generation()) { SaveableStateRegistry(saved.state) { true } }
        saved.registries += registry
        CompositionLocalProvider(LocalSaveableStateRegistry provides registry) {
            if (!saved.hidden) {
                AppTheme {
                    reports.density = LocalDensity.current.density
                    PdfReaderView(
                        document = document,
                        tone = PdfTone.NONE,
                        gutter = Color.Red,
                        highlightsByPage = highlights,
                        highlightFills = IndelibleTheme.colors.highlightFills,
                        restore = restore(),
                        activeSelection = activeSelection(),
                        topInsetPx = 0,
                        onPositioned = { reports.positioned++ },
                        onSample = { reports.samples += it },
                        onSelection = { selection, rect -> reports.selections += selection to rect },
                        onHighlightTapped = { reports.tapped += it },
                        onReaderTap = { reports.readerTaps++ },
                        modifier = Modifier.size(300.dp, height).testTag(VIEW_TAG),
                        zoom = zoom,
                    )
                }
            }
        }
    }
    waitForIdle()
    return reports
}

@OptIn(ExperimentalTestApi::class)
internal fun ComposeUiTest.page(n: Int): SemanticsNodeInteraction = onNodeWithContentDescription("Page $n of 3")

internal fun TouchInjectionScope.at(
    x: Double,
    y: Double,
) = Offset((width * x).toFloat(), (height * y).toFloat())

internal fun TouchInjectionScope.longPressDrag(
    from: Offset,
    through: List<Offset>,
) {
    down(from)
    advanceEventTime(LONG_PRESS_MS)
    moveTo(from + Offset(1f, 0f))
    through.forEach { moveTo(it) }
}
