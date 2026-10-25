package app.indelible.reader.ui.pdf

import androidx.compose.foundation.layout.size
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.toPixelMap
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.test.ComposeUiTest
import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.SemanticsNodeInteraction
import androidx.compose.ui.test.TouchInjectionScope
import androidx.compose.ui.test.captureToImage
import androidx.compose.ui.test.click
import androidx.compose.ui.test.hasScrollToIndexAction
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.performScrollToIndex
import androidx.compose.ui.test.performTouchInput
import androidx.compose.ui.test.runComposeUiTest
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
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNotNull
import kotlin.test.assertTrue

private const val VIEW = "pdfView"
private const val LONG_PRESS_MS = 1_000L

private class Reports {
    var positioned = 0
    var readerTaps = 0
    val samples = mutableListOf<PdfScrollSample>()
    val selections = mutableListOf<Pair<PdfSelection?, SelectionRect?>>()
    val tapped = mutableListOf<String>()
}

private class Scripted(
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

@OptIn(ExperimentalTestApi::class)
class PdfReaderViewTest {
    private fun ComposeUiTest.show(
        document: FakePdfDocument,
        restore: () -> PdfRestoreRequest? = { PdfRestoreRequest(0, 0f, 1) },
        highlights: Map<Int, List<PdfHighlightBox>> = emptyMap(),
        height: Dp = 700.dp,
    ): Reports {
        val reports = Reports()
        setContent {
            AppTheme {
                PdfReaderView(
                    document = document,
                    tone = PdfTone.NONE,
                    gutter = Color.Red,
                    highlightsByPage = highlights,
                    highlightFills = IndelibleTheme.colors.highlightFills,
                    restore = restore(),
                    topInsetPx = 0,
                    onPositioned = { reports.positioned++ },
                    onSample = { reports.samples += it },
                    onSelection = { selection, rect -> reports.selections += selection to rect },
                    onHighlightTapped = { reports.tapped += it },
                    onReaderTap = { reports.readerTaps++ },
                    modifier = Modifier.size(300.dp, height).testTag(VIEW),
                )
            }
        }
        waitForIdle()
        return reports
    }

    private fun ComposeUiTest.page(n: Int): SemanticsNodeInteraction = onNodeWithContentDescription("Page $n of 3")

    private fun TouchInjectionScope.at(
        x: Double,
        y: Double,
    ) = Offset((width * x).toFloat(), (height * y).toFloat())

    private fun TouchInjectionScope.longPressDrag(
        from: Offset,
        through: List<Offset>,
    ) {
        down(from)
        advanceEventTime(LONG_PRESS_MS)
        moveTo(from + Offset(1f, 0f))
        through.forEach { moveTo(it) }
    }

    @Test
    fun pagesStayHiddenUntilRestored() =
        runComposeUiTest {
            var restore by mutableStateOf<PdfRestoreRequest?>(null)
            val reports = show(Scripted().document, restore = { restore })

            val hidden = onNodeWithTag(VIEW).captureToImage().toPixelMap()[150, 100]
            assertEquals(Color.Red, hidden)
            assertEquals(0, reports.positioned)

            restore = PdfRestoreRequest(0, 0f, 1)
            waitForIdle()

            val shown = onNodeWithTag(VIEW).captureToImage().toPixelMap()[150, 100]
            assertEquals(Color.White, shown)
            assertEquals(1, reports.positioned)
        }

    @Test
    fun restoredViewReportsPositionedAtTheSavedPage() =
        runComposeUiTest {
            val reports = show(Scripted().document, restore = { PdfRestoreRequest(1, 0.25f, 1) }, height = 500.dp)

            assertEquals(1, reports.positioned)
            val first = assertNotNull(reports.samples.firstOrNull())
            assertEquals(2, first.page)
            assertTrue(first.pageOffset in 0.23f..0.27f, "offset ${first.pageOffset}")
        }

    @Test
    fun tapOnAHighlightReportsItsId() =
        runComposeUiTest {
            val box = PdfHighlightBox("hlt_1", "yellow", listOf(NormalizedRect(0.1, 0.1, 0.5, 0.05)))
            val reports = show(Scripted().document, highlights = mapOf(0 to listOf(box)))

            page(1).performTouchInput { click(at(0.35, 0.125)) }
            waitForIdle()

            assertEquals(listOf("hlt_1"), reports.tapped)
            assertEquals(0, reports.readerTaps)
        }

    @Test
    fun tapOffAHighlightRevealsTheChrome() =
        runComposeUiTest {
            val box = PdfHighlightBox("hlt_1", "yellow", listOf(NormalizedRect(0.1, 0.1, 0.5, 0.05)))
            val reports = show(Scripted().document, highlights = mapOf(0 to listOf(box)))

            page(1).performTouchInput { click(at(0.5, 0.9)) }
            waitForIdle()

            assertEquals(1, reports.readerTaps)
            assertTrue(reports.tapped.isEmpty())
        }

    @Test
    fun longPressDragReportsTheEngineSelection() =
        runComposeUiTest {
            val reports = show(Scripted().document)

            page(1).performTouchInput {
                longPressDrag(at(0.2, 0.3), listOf(at(0.6, 0.3)))
                up()
            }
            waitForIdle()

            val (selection, rect) = assertNotNull(reports.selections.singleOrNull())
            assertEquals("words to 60", selection?.text)
            assertNotNull(rect)
        }

    @Test
    fun releaseReportsTheSelectionAtTheFinalPoints() =
        runComposeUiTest {
            val gate = CompletableDeferred<Unit>()
            val scripted = Scripted(gate, gateFrom = 0.55)
            val reports = show(scripted.document)

            page(1).performTouchInput {
                longPressDrag(at(0.2, 0.3), listOf(at(0.4, 0.3), at(0.6, 0.3)))
                up()
            }
            waitForIdle()
            assertTrue(reports.selections.isEmpty(), "reported before the final selection arrived")

            gate.complete(Unit)
            waitForIdle()

            val reported = reports.selections.single().first
            assertEquals("words to 60", reported?.text)
        }

    @Test
    fun cancelledDragReportsNothing() =
        runComposeUiTest {
            val reports = show(Scripted().document)

            page(1).performTouchInput {
                longPressDrag(at(0.2, 0.3), listOf(at(0.6, 0.3)))
                cancel()
            }
            waitForIdle()

            assertTrue(reports.selections.isEmpty())
        }

    @Test
    fun dragPastThePageEdgeAsksTheEngineInsideThePage() =
        runComposeUiTest {
            val scripted = Scripted()
            show(scripted.document)

            page(1).performTouchInput {
                longPressDrag(at(0.2, 0.3), listOf(at(1.3, 1.5)))
                up()
            }
            waitForIdle()

            assertTrue(scripted.ends.isNotEmpty())
            assertTrue(scripted.ends.all { it.x in 0.0..1.0 && it.y in 0.0..1.0 }, "ends ${scripted.ends}")
        }

    @Test
    fun eachPageHasItsAccessibilityLabel() =
        runComposeUiTest {
            show(Scripted().document)

            page(1).assertExists()
            page(2).assertExists()
            onNode(hasScrollToIndexAction()).performScrollToIndex(2)
            page(3).assertExists()
        }
}
