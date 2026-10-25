package app.indelible.reader.viewmodel

import app.indelible.core.i18n.UiMessage
import app.indelible.core.offline.ChapterPosition
import app.indelible.core.offline.PagePosition
import app.indelible.core.offline.readingPositionOf
import app.indelible.reader.model.PdfStatus
import app.indelible.reader.model.ReaderDocument
import app.indelible.reader.pdf.PdfDocument
import app.indelible.reader.repository.ReaderRepository
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.reader_pdf_copy_removed
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.launch

/** The open book's or PDF's reading session, whichever the item is; both feed the same success state. */
internal class ReaderSessions(
    private val documentId: String,
    private val repository: ReaderRepository,
    private val scope: CoroutineScope,
    private val progress: ReaderProgressSync,
    private val openPdf: suspend (String) -> PdfDocument,
    private val update: ((ReaderUiState.Success) -> ReaderUiState.Success) -> Unit,
    private val effects: MutableSharedFlow<ReaderEffect>,
) {
    var epub: EpubReaderSession? = null
        private set
    var pdf: PdfReaderSession? = null
        private set

    private var left = false

    fun openBook(
        item: ReaderDocument,
        finished: Boolean,
    ) {
        val resumeAt = item.savedPosition(finished) as? ChapterPosition
        val session = EpubReaderSession(documentId, repository, scope, progress, resumeAt)
        epub = session
        scope.launch {
            session.state.collect { epub ->
                update { it.copy(epub = epub, progress = epub.percent ?: it.progress, toc = epub.contentsPanel()) }
            }
        }
        session.open()
    }

    fun openPdf(
        item: ReaderDocument,
        finished: Boolean,
    ) {
        val resumeAt = item.savedPosition(finished) as? PagePosition
        val percent = if (finished) 0f else item.progressPercent?.toFloat() ?: 0f
        val session = PdfReaderSession(documentId, repository, scope, progress, resumeAt, percent, openPdf)
        pdf = session
        scope.launch {
            session.state.collect { state ->
                update { it.copy(pdf = state, progress = state.percent ?: it.progress) }
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

    fun close() {
        pdf?.close()
    }

    // Remove from device closed the file under the reader: say so once, then leave.
    private suspend fun leaveRemoved() {
        if (left) return
        left = true
        effects.emit(ReaderEffect.ShowSnackbar(UiMessage(Res.string.reader_pdf_copy_removed)))
        effects.emit(ReaderEffect.NavigateBack)
    }
}

private fun ReaderDocument.savedPosition(finished: Boolean) =
    chapterLocator?.takeUnless { finished }?.let { readingPositionOf(it, chapterOffset ?: 0) }
