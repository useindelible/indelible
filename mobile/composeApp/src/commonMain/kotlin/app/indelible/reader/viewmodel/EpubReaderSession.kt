package app.indelible.reader.viewmodel

import app.indelible.core.offline.ChapterPosition
import app.indelible.core.offline.ServerTooOldException
import app.indelible.reader.model.EpubTocEntry
import app.indelible.reader.repository.ReaderRepository
import app.indelible.reader.ui.AnchorScrollRequest
import app.indelible.reader.ui.CharOffsetRequest
import app.indelible.reader.ui.ScrollSample
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.currentCoroutineContext
import kotlinx.coroutines.ensureActive
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.withTimeoutOrNull

enum class EpubStatus {
    LOADING,
    READY,
    CHAPTER_UNAVAILABLE,
    LIST_PENDING,
    NEEDS_REPROCESS,
    NEEDS_REUPLOAD,
    SERVER_TOO_OLD,
    CONNECTION_FAILED,
    EMPTY,
}

/**
 * [percent] is the whole book's (0..100), null before the first sample; [capture] asks the
 * chapter's view for a sample when it changes.
 */
data class EpubReaderState(
    val chapters: List<EpubChapter> = emptyList(),
    val contents: List<EpubTocEntry> = emptyList(),
    val current: Int = 0,
    val html: String? = null,
    val restore: CharOffsetRequest? = null,
    val anchor: AnchorScrollRequest? = null,
    val atChapterEnd: Boolean = false,
    val status: EpubStatus = EpubStatus.LOADING,
    val percent: Float? = null,
    val capture: Long? = null,
)

internal const val CAPTURE_WAIT_MS = 300L

// A chapter that fits reports 0 percent, having no scroll range; its end is its whole.
private const val FULL_CHAPTER = 100f

/**
 * One book, one chapter at a time, in the order of the server's chapter list. It restores the saved
 * character, turns complete samples into progress with a position, and loads one chapter at a time:
 * a later request cancels an earlier one.
 */
class EpubReaderSession internal constructor(
    private val documentId: String,
    private val repository: ReaderRepository,
    private val scope: CoroutineScope,
    private val progress: ReaderProgressSync,
    private val resumeAt: ChapterPosition?,
) {
    private val _state = MutableStateFlow(EpubReaderState())
    val state: StateFlow<EpubReaderState> = _state.asStateFlow()

    private var loadJob: Job? = null
    private var lastTarget: ChapterTarget? = null
    private var tracking = false
    private var nonce = 0L
    private var sampled: CompletableDeferred<Unit>? = null

    fun open() {
        launchLoad {
            _state.value = EpubReaderState()
            val toc = repository.getEpubToc(documentId)
            currentCoroutineContext().ensureActive()
            val book =
                toc.getOrElse { failure ->
                    if (failure is CancellationException) throw failure
                    val status =
                        when (failure) {
                            is ServerTooOldException -> EpubStatus.SERVER_TOO_OLD
                            else -> EpubStatus.CONNECTION_FAILED
                        }
                    _state.update { it.copy(status = status) }
                    return@launchLoad
                }
            val chapters = chapterSequence(book)
            val closed = openingStatus(book, chapters)
            if (closed != null) {
                _state.update { it.copy(contents = book.toc, status = closed) }
                return@launchLoad
            }
            _state.update { it.copy(chapters = chapters, contents = book.toc, status = EpubStatus.LOADING) }
            load(resumeTarget(chapters, book.toc, resumeAt))
        }
    }

    fun goTo(
        index: Int,
        fragment: String? = null,
    ) {
        val current = _state.value
        if (index !in current.chapters.indices) return
        if (index == current.current && current.status == EpubStatus.READY) {
            _state.update { it.copy(restore = null, anchor = null).aimedAt(fragment, offset = 0, ++nonce) }
            return
        }
        launchLoad { load(ChapterTarget(index, offset = 0, fragment)) }
    }

    fun next() = goTo(_state.value.current + 1)

    fun previous() = goTo(_state.value.current - 1)

    fun onTocEntry(id: String) {
        val current = _state.value
        val entry = current.contents.firstOrNull { it.id == id } ?: return
        goTo(current.chapters.indexOfFirst { it.spine == entry.spineIndex }, entry.fragment)
    }

    /** Follows an in-book link: `#frag` in this chapter, or `file.xhtml#frag` in the chapter stored as that file. */
    fun onBookLink(href: String) {
        val file = href.substringBefore('#')
        val fragment = href.substringAfter('#', missingDelimiterValue = "").ifEmpty { null }
        val current = _state.value
        val index =
            if (file.isEmpty()) {
                current.current.takeIf { fragment != null }
            } else {
                current.chapters.indexOfFirst { it.href.substringAfterLast('/') == file.substringAfterLast('/') }
            }
        if (index != null) goTo(index, fragment)
    }

    /** [spine] is the chapter whose view reports; a report from a chapter no longer shown is dropped. */
    fun onPositioned(spine: Int) {
        if (!_state.value.shows(spine)) return
        tracking = true
        _state.update { it.copy(capture = ++nonce) }
    }

    fun onSample(
        spine: Int,
        sample: ScrollSample,
    ) {
        if (!tracking || !_state.value.shows(spine)) return
        val current = _state.value
        val chapterPercent = if (sample.atEnd) FULL_CHAPTER else sample.percent
        val percent = bookPercent(current.current, chapterPercent, current.chapters.size)
        _state.update { it.copy(percent = percent, atChapterEnd = sample.atEnd) }
        if (sample.firstVisibleOffset < 0) return
        val key = current.chapters[current.current].key
        progress.schedule(percent, ChapterPosition(key, sample.firstVisibleOffset))
        sampled?.complete(Unit)
    }

    fun retry() {
        val target = lastTarget
        when (_state.value.status) {
            EpubStatus.CHAPTER_UNAVAILABLE -> if (target != null) launchLoad { load(target) }
            EpubStatus.CONNECTION_FAILED, EpubStatus.SERVER_TOO_OLD, EpubStatus.LIST_PENDING -> open()
            else -> Unit
        }
    }

    fun currentChapterKey(): String? = _state.value.chapterKey()

    /** The server rebuilds the chapter list only when its reprocess job has run, so the book waits for it. */
    fun onReprocessQueued() {
        _state.update { it.copy(status = EpubStatus.LIST_PENDING) }
    }

    /** Asks the chapter for a sample, waits at most [CAPTURE_WAIT_MS] for it, then writes the newest one. */
    suspend fun captureAndFlush() {
        if (tracking) {
            val waiter = CompletableDeferred<Unit>()
            sampled = waiter
            _state.update { it.copy(capture = ++nonce) }
            withTimeoutOrNull(CAPTURE_WAIT_MS) { waiter.await() }
            if (sampled === waiter) sampled = null
        }
        progress.flushNow()
    }

    private suspend fun load(target: ChapterTarget) {
        lastTarget = target
        tracking = false
        _state.update {
            it.copy(
                current = target.index,
                html = null,
                restore = null,
                anchor = null,
                atChapterEnd = false,
                status = EpubStatus.LOADING,
            )
        }
        val spine = _state.value.chapters[target.index].spine
        val html = repository.getEpubChapter(documentId, spine)
        html.exceptionOrNull()?.let { if (it is CancellationException) throw it }
        currentCoroutineContext().ensureActive()
        _state.update { state ->
            html.fold(
                onSuccess = { page ->
                    state.copy(html = page, status = EpubStatus.READY).aimedAt(target.fragment, target.offset, ++nonce)
                },
                onFailure = { state.copy(status = EpubStatus.CHAPTER_UNAVAILABLE) },
            )
        }
    }

    private fun launchLoad(block: suspend () -> Unit) {
        loadJob?.cancel()
        loadJob = scope.launch { block() }
    }
}

private fun EpubReaderState.aimedAt(
    fragment: String?,
    offset: Int,
    nonce: Long,
): EpubReaderState =
    if (fragment != null) {
        copy(anchor = AnchorScrollRequest(fragment, fallbackIndex = -1, nonce = nonce))
    } else {
        copy(restore = CharOffsetRequest(offset, nonce))
    }

private fun EpubReaderState.shows(spine: Int): Boolean {
    val shown = chapters.getOrNull(current)?.spine
    return status == EpubStatus.READY && shown == spine
}
