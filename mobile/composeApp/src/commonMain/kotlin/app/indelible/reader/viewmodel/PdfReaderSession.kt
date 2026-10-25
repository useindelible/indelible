package app.indelible.reader.viewmodel

import app.indelible.core.offline.PAGE_OFFSET_SCALE
import app.indelible.core.offline.PagePosition
import app.indelible.reader.model.PdfHighlightTarget
import app.indelible.reader.model.PdfStatus
import app.indelible.reader.pdf.PdfDocument
import app.indelible.reader.pdf.PdfSelection
import app.indelible.reader.pdf.pdfHighlightTarget
import app.indelible.reader.repository.PdfCopy
import app.indelible.reader.repository.PdfCopyFailure
import app.indelible.reader.repository.ReaderRepository
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlin.math.roundToInt

private const val PERCENT_MAX = 100f

/** [document] is open only while [status] is READY; [download] is the copy's progress while waiting for it. */
data class PdfReaderState(
    val status: PdfStatus = PdfStatus.LOADING,
    val download: Float? = null,
    val document: PdfDocument? = null,
    val restore: PdfRestoreRequest? = null,
    val percent: Float? = null,
)

/**
 * One PDF over its offline copy: it waits for the copy, opens the file with the engine, restores the saved
 * page, turns scroll frames into progress with a page position, and closes the document when the copy goes.
 */
class PdfReaderSession internal constructor(
    private val documentId: String,
    private val repository: ReaderRepository,
    private val scope: CoroutineScope,
    private val progress: ReaderProgressSync,
    private val resumeAt: PagePosition?,
    private val resumePercent: Float,
    private val openPdf: suspend (String) -> PdfDocument,
) {
    private val _state = MutableStateFlow(PdfReaderState())
    val state: StateFlow<PdfReaderState> = _state.asStateFlow()

    private var loadJob: Job? = null
    private var tracking = false
    private var nonce = 0L
    private var closed = false

    fun open() {
        loadJob?.cancel()
        closeDocument()
        tracking = false
        _state.value = PdfReaderState()
        loadJob = scope.launch { repository.pdfCopy(documentId).collect { apply(it) } }
    }

    fun retry() {
        when (_state.value.status) {
            PdfStatus.CONNECTION_FAILED, PdfStatus.NO_SPACE, PdfStatus.DOWNLOAD_FAILED -> open()
            else -> Unit
        }
    }

    /** The view has scrolled to the restore target; samples count from here. */
    fun onPositioned() {
        if (_state.value.status == PdfStatus.READY) tracking = true
    }

    fun onSample(sample: PdfScrollSample) {
        if (!tracking) return
        val percent = (sample.fraction * PERCENT_MAX).coerceIn(0f, PERCENT_MAX)
        _state.update { it.copy(percent = percent) }
        val offset = (sample.pageOffset * PAGE_OFFSET_SCALE).roundToInt().coerceIn(0, PAGE_OFFSET_SCALE)
        progress.schedule(percent, PagePosition(sample.page, offset))
    }

    /** Every frame is a complete sample, so leaving only waits for the newest one to be written. */
    suspend fun captureAndFlush() = progress.flushNow()

    fun highlightTarget(selection: PdfSelection): PdfHighlightTarget? = pdfHighlightTarget(selection)

    fun close() {
        closed = true
        loadJob?.cancel()
        closeDocument()
    }

    private suspend fun apply(copy: PdfCopy) {
        when (copy) {
            is PdfCopy.Downloading ->
                _state.update { it.copy(status = PdfStatus.WAITING_FOR_COPY, download = copy.fraction) }
            is PdfCopy.Ready -> openDocument(copy.path)
            is PdfCopy.Failed -> _state.update { it.copy(status = copy.reason.status(), download = null) }
            PdfCopy.Removed -> {
                closeDocument()
                tracking = false
                _state.update { it.copy(status = PdfStatus.COPY_REMOVED) }
            }
        }
    }

    private suspend fun openDocument(path: String) {
        val document =
            runCatching { openPdf(path) }.getOrElse { error ->
                if (error is CancellationException) throw error
                _state.update { it.copy(status = PdfStatus.UNREADABLE, download = null) }
                return
            }
        if (closed) {
            document.close()
            return
        }
        val restore = pdfResumeTarget(resumeAt, resumePercent, document.pageCount, ++nonce)
        _state.update { it.copy(status = PdfStatus.READY, download = null, document = document, restore = restore) }
    }

    private fun closeDocument() {
        _state.value.document?.close()
        _state.update { it.copy(document = null) }
    }
}

private fun PdfCopyFailure.status(): PdfStatus =
    when (this) {
        PdfCopyFailure.CONNECTION -> PdfStatus.CONNECTION_FAILED
        PdfCopyFailure.NO_SPACE -> PdfStatus.NO_SPACE
        PdfCopyFailure.SERVER -> PdfStatus.DOWNLOAD_FAILED
    }
