package app.indelible.reader.ui.pdf

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.toPixelMap
import androidx.compose.ui.semantics.SemanticsActions
import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.captureToImage
import androidx.compose.ui.test.click
import androidx.compose.ui.test.hasScrollToIndexAction
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performScrollToIndex
import androidx.compose.ui.test.performTouchInput
import androidx.compose.ui.test.runComposeUiTest
import app.indelible.reader.pdf.NormalizedRect
import app.indelible.reader.pdf.PdfHighlightBox
import app.indelible.reader.pdf.PdfSelection
import kotlinx.coroutines.CompletableDeferred
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNotNull
import kotlin.test.assertTrue

@OptIn(ExperimentalTestApi::class)
class PdfReaderViewTest {
    private val box = PdfHighlightBox("hlt_1", "yellow", listOf(NormalizedRect(0.1, 0.1, 0.5, 0.05)))

    @Test
    fun tapOnAHighlightReportsItsId() =
        runComposeUiTest {
            val reports = show(Scripted().document, highlights = mapOf(0 to listOf(box)))

            page(1).performTouchInput { click(at(0.35, 0.125)) }
            waitForIdle()

            assertEquals(listOf("hlt_1"), reports.tapped)
            assertEquals(0, reports.readerTaps)
        }

    @Test
    fun tapOffAHighlightRevealsTheChrome() =
        runComposeUiTest {
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
    fun tapWhileAFinalSelectIsPendingReportsOnlyTheTap() =
        runComposeUiTest {
            val gate = CompletableDeferred<Unit>()
            val reports = show(Scripted(gate, gateFrom = 0.55).document)

            page(1).performTouchInput {
                longPressDrag(at(0.2, 0.3), listOf(at(0.6, 0.3)))
                up()
            }
            waitForIdle()
            page(1).performTouchInput { click(at(0.5, 0.9)) }
            waitForIdle()
            gate.complete(Unit)
            waitForIdle()

            assertEquals(1, reports.readerTaps)
            assertTrue(reports.selections.all { it.first == null }, "selections ${reports.selections}")
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
    fun clearingTheSelectionFromOutsideRemovesTheTint() =
        runComposeUiTest {
            var active by mutableStateOf<PdfSelection?>(null)
            val reports = show(Scripted().document, activeSelection = { active })
            page(1).performTouchInput {
                longPressDrag(at(0.2, 0.3), listOf(at(0.6, 0.3)))
                up()
            }
            waitForIdle()
            active = reports.selections.single().first
            waitForIdle()
            val tinted = onNodeWithTag(VIEW_TAG).captureToImage().toPixelMap()[105, 118]
            assertTrue(tinted != Color.White, "selection not drawn")

            active = null
            waitForIdle()

            assertEquals(Color.White, onNodeWithTag(VIEW_TAG).captureToImage().toPixelMap()[105, 118])
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

    @Test
    fun pageExposesTapAndHighlightsAsAccessibilityActions() =
        runComposeUiTest {
            val reports = show(Scripted().document, highlights = mapOf(0 to listOf(box)))

            page(1).performClick()
            waitForIdle()
            assertEquals(1, reports.readerTaps)

            val config = page(1).fetchSemanticsNode().config
            assertTrue(SemanticsActions.OnClick in config)
            val actions = config[SemanticsActions.CustomActions]
            assertEquals(listOf("Open highlight"), actions.map { it.label })
            actions.single().action()
            waitForIdle()
            assertEquals(listOf("hlt_1"), reports.tapped)
        }
}
