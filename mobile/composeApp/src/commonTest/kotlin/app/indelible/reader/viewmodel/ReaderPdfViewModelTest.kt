package app.indelible.reader.viewmodel

import androidx.lifecycle.ViewModelStore
import app.indelible.core.i18n.UiMessage
import app.indelible.core.offline.PagePosition
import app.indelible.core.offline.ReadingPosition
import app.indelible.reader.model.HighlightColor
import app.indelible.reader.model.PdfHighlightTarget
import app.indelible.reader.model.PdfStatus
import app.indelible.reader.model.ReaderContentMode
import app.indelible.reader.model.TagData
import app.indelible.reader.pdf.NormalizedRect
import app.indelible.reader.repository.PdfCopy
import app.indelible.reader.viewmodel.FakeReaderRepository.Companion.fakeItemDetail
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.reader_pdf_copy_removed
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.toList
import kotlinx.coroutines.launch
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import kotlin.test.AfterTest
import kotlin.test.BeforeTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlin.test.assertTrue

@OptIn(ExperimentalCoroutinesApi::class)
class ReaderPdfViewModelTest {
    private val testDispatcher = UnconfinedTestDispatcher()
    private val repository = FakeReaderRepository()
    private val events = RecordingReadingEventWriter()
    private val document = twentyPages()

    @BeforeTest
    fun setUp() {
        Dispatchers.setMain(testDispatcher)
        repository.getItemResult =
            Result.success(fakeItemDetail(itemType = "pdf").copy(chapterLocator = "page:3", chapterOffset = 2500))
    }

    @AfterTest
    fun tearDown() {
        Dispatchers.resetMain()
    }

    private fun viewModel() = ReaderViewModel("doc_test1", repository, readingEvents = events, openPdf = { document })

    @Test
    fun pdf_item_opens_the_pdf_reader() =
        runTest(testDispatcher) {
            val viewModel = viewModel()
            advanceUntilIdle()

            val state = assertIs<ReaderUiState.Success>(viewModel.uiState.value)
            assertEquals(ReaderContentMode.PDF, state.contentMode)
            assertEquals(PdfStatus.READY, state.pdf?.status)
            assertEquals(2, state.pdf?.restore?.pageIndex)
            assertEquals(0.25f, state.pdf?.restore?.pageOffset)
            assertEquals(TocStatus.NONE, state.toc.status)
            assertTrue(repository.contentCallLog.none { it.startsWith("fetchReadableHtml") })
        }

    @Test
    fun finishedPdfOpensAtTheFirstPage() =
        runTest(testDispatcher) {
            repository.getItemResult =
                Result.success(
                    fakeItemDetail(itemType = "pdf", progressPercent = 100f)
                        .copy(chapterLocator = "page:3", chapterOffset = 2500),
                )
            val viewModel = viewModel()
            advanceUntilIdle()

            val pdf = assertIs<ReaderUiState.Success>(viewModel.uiState.value).pdf
            assertEquals(0, pdf?.restore?.pageIndex)
            assertEquals(0f, pdf?.restore?.pageOffset)
        }

    @Test
    fun toolbarHighlightOnAPdfQueuesAPdfTarget() =
        runTest(testDispatcher) {
            val viewModel = viewModel()
            advanceUntilIdle()

            viewModel.createPdfHighlight(HighlightColor.YELLOW, pdfSelection(text = " quick brown "))
            advanceUntilIdle()

            val target = assertIs<PdfHighlightTarget>(repository.createdTargets.single())
            assertEquals(3, target.page)
            assertEquals("quick brown", target.textSnapshot)
        }

    @Test
    fun toolbarTagOnAPdfOpensTheTagSheetForTheNewHighlight() =
        runTest(testDispatcher) {
            val viewModel = viewModel()
            advanceUntilIdle()
            val tags = listOf(TagData(id = "tag_1", name = "research"))
            repository.listTagsResult = Result.success(tags)
            var opened: Pair<String, List<TagData>>? = null

            viewModel.createPdfHighlight(HighlightColor.YELLOW, pdfSelection()) { id ->
                viewModel.loadTagsForPicker { loaded -> opened = id to loaded }
            }
            advanceUntilIdle()

            assertIs<PdfHighlightTarget>(repository.createdTargets.single())
            assertEquals("hlt_test1" to tags, opened)
        }

    @Test
    fun toolbarNoteOpensTheEditorForTheNewHighlight() =
        runTest(testDispatcher) {
            val viewModel = viewModel()
            advanceUntilIdle()
            var editing: String? = null

            viewModel.createPdfHighlight(HighlightColor.YELLOW, pdfSelection()) { id -> editing = id }
            advanceUntilIdle()

            assertEquals("hlt_test1", editing)
            assertIs<PdfHighlightTarget>(repository.createdTargets.single())
            viewModel.upsertHighlightNote("hlt_test1", "A thought")
            advanceUntilIdle()
            assertEquals("A thought", repository.upsertedNotes.singleOrNull())
        }

    @Test
    fun selectionWithNothingLeftAfterClippingCreatesNothing() =
        runTest(testDispatcher) {
            val viewModel = viewModel()
            advanceUntilIdle()

            val offPage = pdfSelection(rects = listOf(NormalizedRect(1.5, 0.2, 0.5, 0.02)))
            viewModel.createPdfHighlight(HighlightColor.YELLOW, offPage)
            advanceUntilIdle()

            assertTrue(repository.createdTargets.isEmpty())
        }

    @Test
    fun removedCopyLeavesTheReaderWithAMessage() =
        runTest(testDispatcher) {
            val copies = MutableSharedFlow<PdfCopy>()
            repository.pdfCopyFlow = copies
            val viewModel = viewModel()
            val effects = mutableListOf<ReaderEffect>()
            val collecting = launch { viewModel.effects.toList(effects) }
            advanceUntilIdle()
            copies.emit(PdfCopy.Ready(PDF_PATH))
            advanceUntilIdle()

            copies.emit(PdfCopy.Removed)
            advanceUntilIdle()

            val removed = ReaderEffect.ShowSnackbar(UiMessage(Res.string.reader_pdf_copy_removed))
            val expected = listOf(removed, ReaderEffect.NavigateBack)
            assertEquals(expected, effects)
            assertEquals(1, document.closes)
            collecting.cancel()
        }

    @Test
    fun backAfterTheRemovedMessageDoesNotLeaveTwice() =
        runTest(testDispatcher) {
            val copies = MutableSharedFlow<PdfCopy>()
            repository.pdfCopyFlow = copies
            val viewModel = viewModel()
            val effects = mutableListOf<ReaderEffect>()
            val collecting = launch { viewModel.effects.toList(effects) }
            advanceUntilIdle()
            copies.emit(PdfCopy.Ready(PDF_PATH))
            copies.emit(PdfCopy.Removed)
            advanceUntilIdle()

            viewModel.navigateBack()
            advanceUntilIdle()

            assertEquals(1, effects.count { it == ReaderEffect.NavigateBack })
            collecting.cancel()
        }

    @Test
    fun backFlushesThePdfPositionBeforeLeaving() =
        runTest(testDispatcher) {
            val viewModel = viewModel()
            val effects = mutableListOf<ReaderEffect>()
            val collecting = launch { viewModel.effects.toList(effects) }
            advanceUntilIdle()
            val session = requireNotNull(viewModel.pdfSession)
            session.onPositioned()
            session.onSample(PdfScrollSample(page = 9, pageOffset = 0.1f, fraction = 0.405f, scrollTopDp = 0f))
            session.onSample(PdfScrollSample(page = 12, pageOffset = 0.3f, fraction = 0.565f, scrollTopDp = 0f))
            var writtenAtBack: List<ReadingPosition?>? = null
            val watching =
                launch {
                    viewModel.effects.collect {
                        if (it == ReaderEffect.NavigateBack) writtenAtBack = events.positions.toList()
                    }
                }

            viewModel.navigateBack()

            assertEquals(listOf<ReadingPosition?>(PagePosition(9, 1000), PagePosition(12, 3000)), writtenAtBack)
            assertEquals(listOf<ReaderEffect>(ReaderEffect.NavigateBack), effects)
            collecting.cancel()
            watching.cancel()
        }

    @Test
    fun clearingTheViewModelClosesTheDocument() =
        runTest(testDispatcher) {
            val viewModel = viewModel()
            advanceUntilIdle()
            val store = ViewModelStore()
            store.put("reader", viewModel)

            store.clear()

            assertEquals(1, document.closes)
        }

    @Test
    fun book_item_still_opens_the_epub_reader() =
        runTest(testDispatcher) {
            repository.epubTocResult = Result.success(bookToc())
            repository.getItemResult = Result.success(fakeItemDetail(itemType = "book"))
            val viewModel = viewModel()
            advanceUntilIdle()

            val state = assertIs<ReaderUiState.Success>(viewModel.uiState.value)
            assertEquals(ReaderContentMode.EPUB, state.contentMode)
            assertEquals(EpubStatus.READY, state.epub?.status)
        }
}
