package app.indelible.reader.viewmodel

import app.indelible.core.offline.ChapterPosition
import app.indelible.core.offline.PagePosition
import app.indelible.core.offline.readingPositionOf
import app.indelible.reader.model.PdfStatus
import app.indelible.reader.model.ReaderDocument
import app.indelible.reader.pdf.PdfDocument
import app.indelible.reader.repository.ReaderRepository
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.launch

/** Where a session reports: the success-state updater and the effect stream. */
internal class ReaderSessionOutput(
    val update: ((ReaderUiState.Success) -> ReaderUiState.Success) -> Unit,
    val effects: MutableSharedFlow<ReaderEffect>,
)

/** The open book's or PDF's reading session, whichever the item is; both feed the same success state. */
internal class ReaderSessions(
    private val documentId: String,
    private val repository: ReaderRepository,
    private val scope: CoroutineScope,
    private val progress: ReaderProgressSync,
    private val openPdf: suspend (String) -> PdfDocument,
    private val output: ReaderSessionOutput,
) {
    var epub: EpubReaderSession? = null
        private set
    var pdf: PdfReaderSession? = null
        private set

    private var left = false

    // A replaced session must never write into the reader that replaced it.
    private val collectors = mutableListOf<Job>()

    fun openBook(
        item: ReaderDocument,
        finished: Boolean,
    ) {
        close()
        val resumeAt = item.savedPosition(finished) as? ChapterPosition
        val session = EpubReaderSession(documentId, repository, scope, progress, resumeAt)
        epub = session
        collectors += scope.launch {
            session.state.collect { epub ->
                output.update {
                    it.copy(epub = epub, progress = epub.percent ?: it.progress, toc = epub.contentsPanel())
                }
            }
        }
        session.open()
    }

    fun openPdf(
        item: ReaderDocument,
        finished: Boolean,
        percent: Float,
    ) {
        close()
        val resumeAt = item.savedPosition(finished) as? PagePosition
        val session = PdfReaderSession(documentId, repository, scope, progress, PdfResume(resumeAt, percent), openPdf)
        pdf = session
        collectors += scope.launch {
            session.state.collect { state ->
                output.update { it.copy(pdf = state, progress = state.percent ?: it.progress) }
                if (state.status == PdfStatus.COPY_REMOVED) leaveRemoved()
            }
        }
        session.open()
    }

    /** Writes the newest position: a book asks its chapter first, a PDF has it, anything else just flushes. */
    suspend fun captureAndFlush() {
        when {
            epub != null -> epub?.captureAndFlush()
            pdf != null -> pdf?.captureAndFlush()
            else -> progress.flush()
        }
    }

    /** True once the removal has queued its own NavigateBack. */
    val leaving: Boolean get() = left

    /** Asks the open book or PDF for its copy again. */
    fun retry() {
        pdf?.retry() ?: epub?.retry()
    }

    fun close() {
        collectors.forEach { it.cancel() }
        collectors.clear()
        pdf?.close()
        epub?.close()
        pdf = null
        epub = null
    }

    // Remove from device closed the file under the reader: leave once.
    private suspend fun leaveRemoved() {
        if (left) return
        left = true
        output.effects.emit(ReaderEffect.NavigateBack)
    }
}

private fun ReaderDocument.savedPosition(finished: Boolean) =
    chapterLocator?.takeUnless { finished }?.let { readingPositionOf(it, chapterOffset ?: 0) }
