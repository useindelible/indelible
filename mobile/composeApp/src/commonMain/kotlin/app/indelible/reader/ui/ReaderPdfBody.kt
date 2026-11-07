package app.indelible.reader.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.unit.dp
import app.indelible.reader.model.HighlightData
import app.indelible.reader.model.PERCENT_MAX
import app.indelible.reader.model.PdfStatus
import app.indelible.reader.pdf.pdfHighlightsByPage
import app.indelible.reader.pdf.toneFor
import app.indelible.reader.ui.components.PdfStatusContent
import app.indelible.reader.ui.pdf.PdfReaderView
import app.indelible.reader.viewmodel.PdfReaderSession
import app.indelible.reader.viewmodel.PdfReaderState
import app.indelible.reader.viewmodel.ReaderUiState
import app.indelible.ui.theme.IndelibleTheme
import app.indelible.ui.theme.canvasFor

private val NO_RECT = SelectionRect(0f, 0f, 0f, 0f)

/** What the reader shows for a PDF: its pages, a spinner while the copy is checked, or why it is not open. */
@Composable
internal fun ReaderPdfBody(
    state: ReaderUiState.Success,
    pdf: PdfReaderState,
    session: PdfReaderSession,
    selectedText: SelectedTextInfo?,
    topChromeInsetPx: Int,
    onSelectedTextChanged: (SelectedTextInfo?) -> Unit,
    onTappedHighlightChanged: (HighlightData?) -> Unit,
    onReaderTap: () -> Unit,
    handleScrollProgress: (Float, Float) -> Unit,
    onOpenStorage: () -> Unit,
    onRetry: () -> Unit,
    modifier: Modifier = Modifier,
) {
    LaunchedEffect(pdf.status, pdf.document) {
        onSelectedTextChanged(null)
        onTappedHighlightChanged(null)
    }
    val background = state.preferences.background
    val canvas = IndelibleTheme.colors.readerCanvas.canvasFor(background)
    val document = pdf.document
    val highlights = state.highlights
    val byPage = remember(highlights) { pdfHighlightsByPage(highlights) }
    when {
        pdf.status == PdfStatus.READY && document != null ->
            PdfReaderView(
                document = document,
                tone = toneFor(background),
                gutter = canvas,
                highlightsByPage = byPage,
                highlightFills = IndelibleTheme.colors.highlightFills,
                restore = pdf.restore,
                activeSelection = selectedText?.pdf,
                topInsetPx = with(LocalDensity.current) { topChromeInsetPx.dp.roundToPx() },
                onPositioned = session::onPositioned,
                onSample = { sample ->
                    handleScrollProgress(sample.fraction * PERCENT_MAX, sample.scrollTopDp)
                    session.onSample(sample)
                },
                onSelection = { selection, rect ->
                    onSelectedTextChanged(selection?.let { SelectedTextInfo(it.text, 0, 0, rect ?: NO_RECT, pdf = it) })
                },
                onHighlightTapped = { id -> onTappedHighlightChanged(highlights.find { it.id == id }) },
                onReaderTap = onReaderTap,
                modifier = modifier,
            )

        pdf.status == PdfStatus.LOADING ->
            Box(modifier.background(canvas), contentAlignment = Alignment.Center) { CircularProgressIndicator() }

        else ->
            PdfStatusContent(
                status = pdf.status,
                downloadFraction = pdf.download,
                onRetry = onRetry,
                onManageStorage = onOpenStorage,
                modifier = modifier,
            )
    }
}
