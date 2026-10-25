package app.indelible.reader.viewmodel

import app.indelible.core.offline.PagePosition
import app.indelible.core.offline.ReadingPosition
import app.indelible.reader.pdf.FakePdfDocument
import app.indelible.reader.pdf.NormalizedRect
import app.indelible.reader.pdf.PdfDocument
import app.indelible.reader.pdf.PdfPageSize
import app.indelible.reader.pdf.PdfSelection
import app.indelible.reader.repository.PdfCopy
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.onStart
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.TestScope

internal const val PDF_PATH = "/offline/doc_pdf/1/document.pdf"

internal fun twentyPages() = FakePdfDocument(List(20) { PdfPageSize(612.0, 792.0) })

internal fun pdfSelection(
    text: String = "quick brown",
    rects: List<NormalizedRect> = listOf(NormalizedRect(0.1, 0.2, 0.5, 0.02)),
) = PdfSelection(pageIndex = 2, text = text, startChar = 4, endChar = 15, rects = rects)

internal class PdfHarness(
    val repository: FakeReaderRepository,
    val written: MutableList<Pair<Float, ReadingPosition?>>,
    val session: PdfReaderSession,
    val document: FakePdfDocument,
) {
    val state: PdfReaderState get() = session.state.value
    var copyRequests = 0
}

/** A session over [copy]; [opener] stands in for the engine and gets [document] by default. */
internal fun TestScope.pdfHarness(
    resumeAt: PagePosition? = null,
    percent: Float = 0f,
    copy: Flow<PdfCopy> = flowOf(PdfCopy.Ready(PDF_PATH)),
    document: FakePdfDocument = twentyPages(),
    opener: suspend (String) -> PdfDocument = { document },
): PdfHarness {
    val repository = FakeReaderRepository()
    val written = mutableListOf<Pair<Float, ReadingPosition?>>()
    val scope = CoroutineScope(StandardTestDispatcher(testScheduler) + SupervisorJob())
    val progress = ReaderProgressSync(scope) { p, position -> written += p to position }
    lateinit var harness: PdfHarness
    repository.pdfCopyFlow = copy.onStart { harness.copyRequests++ }
    val session = PdfReaderSession("doc_pdf", repository, scope, progress, resumeAt, percent, opener)
    harness = PdfHarness(repository, written, session, document)
    return harness
}
