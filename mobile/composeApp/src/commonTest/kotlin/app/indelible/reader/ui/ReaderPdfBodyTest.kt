package app.indelible.reader.ui

import androidx.compose.foundation.layout.size
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.test.ComposeUiTest
import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.click
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performTouchInput
import androidx.compose.ui.test.runComposeUiTest
import androidx.compose.ui.unit.dp
import app.indelible.reader.model.HighlightData
import app.indelible.reader.model.HighlightLocator
import app.indelible.reader.model.PdfRect
import app.indelible.reader.model.PdfStatus
import app.indelible.reader.model.ReaderContentMode
import app.indelible.reader.model.ReaderPreferences
import app.indelible.reader.pdf.FakePdfDocument
import app.indelible.reader.pdf.NormalizedRect
import app.indelible.reader.pdf.PdfPageSize
import app.indelible.reader.pdf.PdfSelection
import app.indelible.reader.viewmodel.FakeReaderRepository
import app.indelible.reader.viewmodel.FakeReaderRepository.Companion.fakeHighlight
import app.indelible.reader.viewmodel.FakeReaderRepository.Companion.fakeItemDetail
import app.indelible.reader.viewmodel.PdfReaderSession
import app.indelible.reader.viewmodel.PdfReaderState
import app.indelible.reader.viewmodel.PdfResume
import app.indelible.reader.viewmodel.ReaderProgressSync
import app.indelible.reader.viewmodel.ReaderUiState
import app.indelible.ui.theme.AppTheme
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNotNull
import kotlin.test.assertTrue

private const val LONG_PRESS_MS = 1_000L

private val selection = PdfSelection(0, "quick brown", 4, 15, listOf(NormalizedRect(0.1, 0.3, 0.5, 0.02)))
private val pdfHighlight =
    fakeHighlight(id = "hlt_pdf").copy(
        locator =
            HighlightLocator(
                type = "pdf",
                page = 1,
                x = 0.1,
                y = 0.1,
                width = 0.5,
                height = 0.05,
                textSnapshot = "words",
                rects = listOf(PdfRect(0.1, 0.1, 0.5, 0.05)),
            ),
    )

private class Reports {
    val selected = mutableListOf<SelectedTextInfo?>()
    val tapped = mutableListOf<HighlightData?>()
    val progress = mutableListOf<Pair<Float, Float>>()
}

@OptIn(ExperimentalTestApi::class)
class ReaderPdfBodyTest {
    private val document =
        FakePdfDocument(List(3) { PdfPageSize(612.0, 792.0) }, selectionFor = { _, _, _ -> selection })
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.Unconfined)
    private val progress = ReaderProgressSync(scope) { _, _ -> }
    private val session =
        PdfReaderSession("doc_test1", FakeReaderRepository(), scope, progress, PdfResume(null, 0f)) { document }

    /** Opens the session over the fake copy, so samples count once the view reports its position. */
    private fun ready(): PdfReaderState {
        session.open()
        return session.state.value.also { check(it.status == PdfStatus.READY) }
    }

    private fun ComposeUiTest.show(
        pdf: PdfReaderState,
        highlights: List<HighlightData> = emptyList(),
    ): Reports {
        val reports = Reports()
        val state =
            ReaderUiState.Success(
                item = fakeItemDetail(itemType = "pdf"),
                htmlContent = null,
                contentMode = ReaderContentMode.PDF,
                highlights = highlights,
                progress = 0f,
                preferences = ReaderPreferences(),
                pdf = pdf,
            )
        setContent {
            AppTheme {
                ReaderPdfBody(
                    state = state,
                    pdf = pdf,
                    session = session,
                    selectedText = null,
                    topChromeInsetPx = 0,
                    onSelectedTextChanged = { reports.selected += it },
                    onTappedHighlightChanged = { reports.tapped += it },
                    onReaderTap = {},
                    handleScrollProgress = { percent, top -> reports.progress += percent to top },
                    onOpenStorage = {},
                    modifier = Modifier.size(300.dp, 700.dp),
                )
            }
        }
        waitForIdle()
        return reports
    }

    @Test
    fun readyPdfReportsASelectionWithItsPdfGeometry() =
        runComposeUiTest {
            val reports = show(ready())

            onNodeWithContentDescription("Page 1 of 3").performTouchInput {
                down(Offset(width * 0.2f, height * 0.3f))
                advanceEventTime(LONG_PRESS_MS)
                moveTo(Offset(width * 0.6f, height * 0.3f))
                up()
            }
            waitForIdle()

            val selected = assertNotNull(reports.selected.last())
            assertEquals(selection, selected.pdf)
            assertEquals("quick brown", selected.text)
            assertEquals(0, selected.startOffset)
        }

    @Test
    fun tappedHighlightComesFromTheReaderState() =
        runComposeUiTest {
            val reports = show(ready(), highlights = listOf(pdfHighlight))

            onNodeWithContentDescription("Page 1 of 3").performTouchInput {
                click(Offset(width * 0.35f, height * 0.125f))
            }
            waitForIdle()

            assertEquals(listOf<HighlightData?>(pdfHighlight), reports.tapped.filterNotNull())
        }

    @Test
    fun samplesReachTheScreenAndTheSession() =
        runComposeUiTest {
            val reports = show(ready())

            assertTrue(reports.progress.isNotEmpty())
            assertEquals(0f, session.state.value.percent)
        }

    @Test
    fun waitingPdfShowsTheDownloadProgress() =
        runComposeUiTest {
            show(PdfReaderState(status = PdfStatus.WAITING_FOR_COPY, download = 0.4f))

            onNodeWithText("Getting this PDF").assertIsDisplayed()
        }
}
