package app.indelible.reader.viewmodel

import app.indelible.core.offline.ChapterPosition
import app.indelible.reader.model.EpubChapterList
import app.indelible.reader.model.HighlightColor
import app.indelible.reader.model.ReaderContentMode
import app.indelible.reader.viewmodel.FakeReaderRepository.Companion.fakeItemDetail
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.awaitCancellation
import kotlinx.coroutines.flow.toList
import kotlinx.coroutines.launch
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runCurrent
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import kotlin.test.AfterTest
import kotlin.test.BeforeTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlin.test.assertTrue

@OptIn(ExperimentalCoroutinesApi::class)
class ReaderEpubViewModelTest {
    private val testDispatcher = UnconfinedTestDispatcher()
    private val repository = FakeReaderRepository()
    private val events = RecordingReadingEventWriter()

    @BeforeTest
    fun setUp() {
        Dispatchers.setMain(testDispatcher)
        repository.epubTocResult = Result.success(bookToc())
        repository.getItemResult =
            Result.success(fakeItemDetail(itemType = "book").copy(chapterLocator = "e5", chapterOffset = 7))
    }

    @AfterTest
    fun tearDown() {
        Dispatchers.resetMain()
    }

    @Test
    fun book_item_opens_the_epub_reader() =
        runTest(testDispatcher) {
            val viewModel = ReaderViewModel("doc_test1", repository, readingEvents = events)
            advanceUntilIdle()

            val state = assertIs<ReaderUiState.Success>(viewModel.uiState.value)
            assertEquals(ReaderContentMode.EPUB, state.contentMode)
            assertEquals(EpubStatus.READY, state.epub?.status)
            assertEquals("<p>chapter 5</p>", state.epub?.html)
            assertEquals(7, state.epub?.restore?.offset)
            assertTrue(repository.contentCallLog.none { it.startsWith("fetchReadableHtml") })
        }

    @Test
    fun a_finished_book_opens_at_its_first_chapter() =
        runTest(testDispatcher) {
            repository.getItemResult =
                Result.success(
                    fakeItemDetail(itemType = "book", progressPercent = 100f)
                        .copy(chapterLocator = "e5", chapterOffset = 7),
                )
            val viewModel = ReaderViewModel("doc_test1", repository, readingEvents = events)
            advanceUntilIdle()

            val epub = assertIs<ReaderUiState.Success>(viewModel.uiState.value).epub
            assertEquals(0, epub?.current)
            assertEquals(0, epub?.restore?.offset ?: 0)
        }

    @Test
    fun a_book_that_could_not_be_opened_can_be_tried_again() =
        runTest(testDispatcher) {
            repository.getItemResults =
                listOf(Result.failure(IllegalStateException("no answer")), repository.getItemResult)
            val viewModel = ReaderViewModel("doc_test1", repository, readingEvents = events)
            advanceUntilIdle()
            assertIs<ReaderUiState.Error>(viewModel.uiState.value)

            viewModel.retryOpen()
            advanceUntilIdle()

            assertEquals(EpubStatus.READY, assertIs<ReaderUiState.Success>(viewModel.uiState.value).epub?.status)
        }

    @Test
    fun epub_open_emits_no_scroll_to_percent() =
        runTest(testDispatcher) {
            val viewModel = ReaderViewModel("doc_test1", repository, readingEvents = events)
            val effects = mutableListOf<ReaderEffect>()
            val collecting = launch { viewModel.effects.toList(effects) }
            advanceUntilIdle()

            viewModel.onContentLoaded()
            advanceUntilIdle()

            assertTrue(effects.none { it is ReaderEffect.ScrollToPercent })
            collecting.cancel()
        }

    @Test
    fun toolbar_highlight_in_a_book_carries_the_chapter() =
        runTest(testDispatcher) {
            val viewModel = ReaderViewModel("doc_test1", repository, readingEvents = events)
            advanceUntilIdle()

            viewModel.createHighlight(HighlightColor.YELLOW, "text", startOffset = 1, endOffset = 5)
            advanceUntilIdle()

            assertEquals("e5", repository.createdRanges.single().chapter)
        }

    @Test
    fun highlight_while_a_chapter_loads_is_refused() =
        runTest(testDispatcher) {
            repository.epubChapterResult = { awaitCancellation() }
            val viewModel = ReaderViewModel("doc_test1", repository, readingEvents = events)
            advanceUntilIdle()

            viewModel.createHighlight(HighlightColor.YELLOW, "text", startOffset = 1, endOffset = 5)
            advanceUntilIdle()

            assertEquals(0, repository.createHighlightCallCount)
        }

    @Test
    fun contents_list_the_book_and_open_its_entries() =
        runTest(testDispatcher) {
            val viewModel = ReaderViewModel("doc_test1", repository, readingEvents = events)
            advanceUntilIdle()
            val toc = assertIs<ReaderUiState.Success>(viewModel.uiState.value).toc
            assertEquals(listOf("e0", "e5", "e5b"), toc.entries.map { it.id })
            assertEquals(1, toc.activeIndex)

            viewModel.onTocEntryTapped(toc.entries.first())
            advanceUntilIdle()

            assertEquals(0, assertIs<ReaderUiState.Success>(viewModel.uiState.value).epub?.current)
        }

    @Test
    fun reprocessing_a_book_shows_it_is_being_prepared() =
        runTest(testDispatcher) {
            repository.epubTocResult = Result.success(bookToc(chapterList = EpubChapterList.NEEDS_REPROCESS))
            val viewModel = ReaderViewModel("doc_test1", repository, readingEvents = events)
            advanceUntilIdle()
            assertEquals(TocStatus.NONE, assertIs<ReaderUiState.Success>(viewModel.uiState.value).toc.status)

            viewModel.reprocessBook()
            advanceUntilIdle()

            assertEquals(EpubStatus.LIST_PENDING, assertIs<ReaderUiState.Success>(viewModel.uiState.value).epub?.status)
        }

    @Test
    fun stopping_a_book_writes_its_position() =
        runTest(testDispatcher) {
            val viewModel = ReaderViewModel("doc_test1", repository, readingEvents = events)
            advanceUntilIdle()
            val session = requireNotNull(viewModel.epubSession)
            session.onPositioned(5)
            session.onSample(5, sample(percent = 10f, offset = 1))

            viewModel.onStop()
            session.onSample(5, sample(percent = 20f, offset = 3))
            runCurrent()

            assertEquals(listOf<ChapterPosition?>(ChapterPosition("e5", 1), ChapterPosition("e5", 3)), events.positions)
        }

    @Test
    fun book_ignores_article_scroll_progress() =
        runTest(testDispatcher) {
            val viewModel = ReaderViewModel("doc_test1", repository, readingEvents = events)
            advanceUntilIdle()

            viewModel.onScrollProgress(60f)
            advanceUntilIdle()

            assertTrue(events.progressPercents.isEmpty())
        }

    @Test
    fun back_in_a_book_writes_the_position_before_navigating() =
        runTest(testDispatcher) {
            val viewModel = ReaderViewModel("doc_test1", repository, readingEvents = events)
            val effects = mutableListOf<ReaderEffect>()
            val collecting = launch { viewModel.effects.toList(effects) }
            advanceUntilIdle()
            val session = requireNotNull(viewModel.epubSession)
            session.onPositioned(5)

            viewModel.navigateBack()
            viewModel.navigateBack()
            session.onSample(5, sample(percent = 50f, offset = 11))
            advanceUntilIdle()

            assertEquals(listOf<ChapterPosition?>(ChapterPosition("e5", 11)), events.positions)
            assertEquals(listOf<ReaderEffect>(ReaderEffect.NavigateBack), effects)
            collecting.cancel()
        }
}
