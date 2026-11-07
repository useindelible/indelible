package app.indelible.reader.viewmodel

import app.indelible.api.generated.models.EpubMetadata
import app.indelible.api.generated.models.EpubStoredChapter
import app.indelible.core.offline.ChapterPosition
import app.indelible.reader.model.EpubChapterList
import app.indelible.reader.model.EpubToc
import app.indelible.reader.model.EpubTocEntry
import app.indelible.reader.ui.ScrollSample
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.TestScope
import kotlinx.coroutines.test.advanceUntilIdle

internal fun tocEntry(
    id: String,
    spine: Int,
    title: String = "T$id",
    fragment: String? = null,
) = EpubTocEntry(
    depth = 0,
    fragment = fragment,
    id = id,
    spineIndex = spine,
    startPage = 1,
    title = title,
    wordCount = 10,
)

internal fun storedChapter(
    spine: Int,
    title: String = "",
) = EpubStoredChapter(href = "c$spine.xhtml", spineIndex = spine, title = title)

/** A book whose chapters are spines 0, 2 and 5; spine 2 has no outline entry. */
internal fun bookToc(
    chapters: List<EpubStoredChapter>? = listOf(storedChapter(0), storedChapter(2, "Server two"), storedChapter(5)),
    toc: List<EpubTocEntry> = listOf(tocEntry("e0", 0), tocEntry("e5", 5), tocEntry("e5b", 5, fragment = "later")),
    chapterList: EpubChapterList = EpubChapterList.READY,
) = EpubToc(
    chapterList = chapterList,
    metadata = EpubMetadata(chapters = chapters, estimatedPages = 3, totalChapters = 3, totalWords = 30),
    toc = toc,
)

internal class EpubHarness(
    val repository: FakeReaderRepository,
    val written: MutableList<Pair<Float, ChapterPosition?>>,
    val session: EpubReaderSession,
) {
    val state: EpubReaderState get() = session.state.value

    private val spine: Int get() = state.chapters.getOrNull(state.current)?.spine ?: -1

    fun onPositioned() = session.onPositioned(spine)

    fun onSample(sample: ScrollSample) = session.onSample(spine, sample)
}

internal fun TestScope.epubHarness(
    resumeAt: ChapterPosition? = null,
    toc: Result<EpubToc> = Result.success(bookToc()),
): EpubHarness {
    val repository = FakeReaderRepository().apply { epubTocResult = toc }
    val written = mutableListOf<Pair<Float, ChapterPosition?>>()
    val scope = CoroutineScope(StandardTestDispatcher(testScheduler) + SupervisorJob())
    val progress = ReaderProgressSync(scope) { percent, position -> written += percent to position }
    return EpubHarness(repository, written, EpubReaderSession("doc_book", repository, scope, progress, resumeAt))
}

/** Opens the book and lets its first chapter report that it is positioned. */
@OptIn(ExperimentalCoroutinesApi::class)
internal fun TestScope.openedAndPositioned(h: EpubHarness) {
    h.session.open()
    advanceUntilIdle()
    h.onPositioned()
}

internal fun sample(
    percent: Float,
    offset: Int = -1,
    atEnd: Boolean = false,
) = ScrollSample(percent, scrollTopDp = 0f, firstVisibleOffset = offset, atEnd = atEnd)
