package app.indelible.reader.ui.pdf

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.toPixelMap
import androidx.compose.ui.test.ComposeUiTest
import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.captureToImage
import androidx.compose.ui.test.hasScrollToIndexAction
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.performScrollToIndex
import androidx.compose.ui.test.performTouchInput
import androidx.compose.ui.test.runComposeUiTest
import androidx.compose.ui.unit.dp
import app.indelible.reader.viewmodel.PdfRestoreRequest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNotNull
import kotlin.test.assertNull
import kotlin.test.assertTrue

@OptIn(ExperimentalTestApi::class)
class PdfReaderViewLayoutTest {
    @Test
    fun pagesStayHiddenUntilRestored() =
        runComposeUiTest {
            var restore by mutableStateOf<PdfRestoreRequest?>(null)
            val reports = show(Scripted().document, restore = { restore })

            val hidden = onNodeWithTag(VIEW_TAG).captureToImage().toPixelMap()[150, 100]
            assertEquals(Color.Red, hidden)
            assertEquals(0, reports.positioned)

            restore = PdfRestoreRequest(0, 0f, 1)
            waitForIdle()

            val shown = onNodeWithTag(VIEW_TAG).captureToImage().toPixelMap()[150, 100]
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
    fun recreatedViewKeepsItsScrolledPlace() =
        runComposeUiTest {
            var generation by mutableStateOf(0)
            val saved = SavedViews()
            val reports = show(Scripted().document, height = 300.dp, saved = saved, generation = { generation })
            onNode(hasScrollToIndexAction()).performScrollToIndex(2)
            waitForIdle()
            assertEquals(3, reports.samples.last().page)

            saved.state = saved.registries.last().performSave()
            saved.hidden = true
            waitForIdle()
            generation++
            saved.hidden = false
            waitForIdle()

            assertEquals(3, reports.samples.last().page)
            assertEquals(2, reports.positioned)
        }

    @Test
    fun resizedViewKeepsItsPlaceInThePage() =
        runComposeUiTest {
            var width by mutableStateOf(300.dp)
            val restore = PdfRestoreRequest(1, 0.5f, 1)
            val reports = show(Scripted().document, restore = { restore }, height = 300.dp, width = { width })
            assertTrue(reports.samples.last().pageOffset in 0.48f..0.52f, "offset ${reports.samples.last().pageOffset}")

            width = 600.dp
            waitForIdle()

            val sample = reports.samples.last()
            assertEquals(2, sample.page)
            assertTrue(sample.pageOffset in 0.48f..0.52f, "offset ${sample.pageOffset}")
        }

    @Test
    fun recreatedViewAtAnotherWidthKeepsItsPlaceInThePage() =
        runComposeUiTest {
            var width by mutableStateOf(300.dp)
            var generation by mutableStateOf(0)
            val saved = SavedViews()
            val reports =
                show(
                    Scripted().document,
                    restore = { PdfRestoreRequest(1, 0.5f, 1) },
                    height = 300.dp,
                    width = { width },
                    saved = saved,
                    generation = { generation },
                )
            val first = assertNotNull(reports.samples.firstOrNull())
            assertTrue(first.pageOffset in 0.48f..0.52f, "offset ${first.pageOffset}")

            saved.state = saved.registries.last().performSave()
            saved.hidden = true
            waitForIdle()
            width = 600.dp
            generation++
            saved.hidden = false
            waitForIdle()

            val sample = reports.samples.last()
            assertEquals(2, sample.page)
            assertTrue(sample.pageOffset in 0.48f..0.52f, "offset ${sample.pageOffset}")
        }

    @Test
    fun samplesAfterAZoomUseTheZoomedHeights() =
        runComposeUiTest {
            val zoom = PdfZoomState()
            val reports = show(Scripted().document, height = 300.dp, zoom = zoom)

            zoom.zoom = 2f
            waitForIdle()
            onNode(hasScrollToIndexAction()).performScrollToIndex(1)
            waitForIdle()

            val sample = reports.samples.last()
            assertEquals(2, sample.page)
            val zoomedPageAndGap = (776 + 8) / reports.density
            val near = (zoomedPageAndGap - 2f)..(zoomedPageAndGap + 2f)
            assertTrue(sample.scrollTopDp in near, "top ${sample.scrollTopDp}")
        }

    private fun ComposeUiTest.pinchOut() =
        onNodeWithTag(VIEW_TAG).performTouchInput {
            down(0, Offset(125f, 300f))
            down(1, Offset(175f, 300f))
            advanceEventTime(50)
            moveTo(0, Offset(100f, 300f))
            moveTo(1, Offset(200f, 300f))
            advanceEventTime(50)
            up(0)
            up(1)
        }

    @Test
    fun stillTwoFingerTouchWhileZoomedEndsCleanly() =
        runComposeUiTest {
            val zoom = PdfZoomState()
            show(Scripted().document, zoom = zoom)
            pinchOut()
            waitForIdle()

            onNodeWithTag(VIEW_TAG).performTouchInput {
                down(0, Offset(100f, 400f))
                down(1, Offset(200f, 400f))
                advanceEventTime(50)
                up(0)
                up(1)
            }
            waitForIdle()

            assertEquals(2f, zoom.zoom)
            assertNull(zoom.preview)
        }

    @Test
    fun recreatedViewKeepsItsZoom() =
        runComposeUiTest {
            var generation by mutableStateOf(0)
            val saved = SavedViews()
            show(Scripted().document, saved = saved, generation = { generation })
            pinchOut()
            waitForIdle()
            assertTrue(page(1).fetchSemanticsNode().size.width in 598..602)

            saved.state = saved.registries.last().performSave()
            saved.hidden = true
            waitForIdle()
            generation++
            saved.hidden = false
            waitForIdle()

            val node = page(1).fetchSemanticsNode()
            assertTrue(node.size.width in 598..602, "width ${node.size.width}")
            assertTrue(node.positionInRoot.x in -152f..-148f, "left ${node.positionInRoot.x}")
        }

    @Test
    fun pinchZoomsAroundTheFingers() =
        runComposeUiTest {
            val zoom = PdfZoomState()
            show(Scripted().document, zoom = zoom)

            pinchOut()
            waitForIdle()

            assertEquals(2f, zoom.zoom)
            val node = page(1).fetchSemanticsNode()
            assertTrue(node.size.width in 598..602, "width ${node.size.width}")
            assertTrue(node.positionInRoot.x in -152f..-148f, "left ${node.positionInRoot.x}")
            assertTrue(node.positionInRoot.y in -302f..-298f, "top ${node.positionInRoot.y}")
        }
}
