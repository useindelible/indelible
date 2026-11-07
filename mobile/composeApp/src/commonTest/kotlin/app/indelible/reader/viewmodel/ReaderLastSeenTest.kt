package app.indelible.reader.viewmodel

import app.indelible.reader.model.HighlightData
import app.indelible.reader.model.PdfStatus
import app.indelible.reader.model.ReaderDocument
import app.indelible.reader.repository.PdfCopy
import app.indelible.reader.repository.PdfCopyFailure
import app.indelible.reader.viewmodel.FakeReaderRepository.Companion.fakeHighlight
import app.indelible.reader.viewmodel.FakeReaderRepository.Companion.fakeItemDetail
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.launch
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import kotlinx.io.IOException
import kotlin.test.AfterTest
import kotlin.test.BeforeTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertIs
import kotlin.test.assertTrue

@OptIn(ExperimentalCoroutinesApi::class)
class ReaderLastSeenTest {
    private val testDispatcher = UnconfinedTestDispatcher()
    private val repository = FakeReaderRepository()
    private val document = twentyPages()
    private val noAnswer = flowOf(PdfCopy.Failed(PdfCopyFailure.CONNECTION))
    private val events = RecordingReadingEventWriter()

    @BeforeTest
    fun setUp() {
        Dispatchers.setMain(testDispatcher)
        repository.getItemResult = Result.success(lastSeen("pdf"))
        repository.pdfCopyFlow = noAnswer
    }

    @AfterTest
    fun tearDown() {
        Dispatchers.resetMain()
    }

    private fun lastSeen(type: String): ReaderDocument =
        fakeItemDetail(itemType = type, progressPercent = 40f, readableReady = false)
            .copy(libraryEntryId = null, availableAssets = emptyList(), lastSeen = true)

    private fun viewModel() =
        ReaderViewModel("doc_test1", repository, readingEvents = events, openPdf = { document })

    private fun ReaderViewModel.success() = assertIs<ReaderUiState.Success>(uiState.value)

    @Test
    fun lastSeenPdfShowsTheConnectionScreenAndStaysQuiet() =
        runTest(testDispatcher) {
            val viewModel = viewModel()
            val effects = mutableListOf<ReaderEffect>()
            backgroundScope.launch { viewModel.effects.collect { effects += it } }
            advanceUntilIdle()

            assertEquals(PdfStatus.CONNECTION_FAILED, viewModel.success().pdf?.status)
            assertTrue(effects.none { it is ReaderEffect.ShowSnackbar })
        }

    @Test
    fun retryOnALastSeenPdfReloadsTheWholeReader() =
        runTest(testDispatcher) {
            val server = fakeItemDetail(id = "lib_1", itemType = "pdf", progressPercent = 60f)
            repository.getItemResults = listOf(Result.success(lastSeen("pdf")), Result.success(server))
            val viewModel = viewModel()
            advanceUntilIdle()
            repository.pdfCopyFlow = flowOf(PdfCopy.Ready(PDF_PATH))
            repository.listHighlightsResult = Result.success(listOf<HighlightData>(fakeHighlight()))

            viewModel.retryOpen()
            advanceUntilIdle()

            val state = viewModel.success()
            assertFalse(state.item.lastSeen)
            assertEquals(60, state.item.progressPercent)
            assertEquals(PdfStatus.READY, state.pdf?.status)
            assertEquals(12, state.pdf?.restore?.pageIndex)
            assertEquals(1, state.highlights.size)
            assertEquals("lib_1", repository.lastGetItemTagsId)
        }

    @Test
    fun repeatedRetriesLeaveOneSessionCollecting() =
        runTest(testDispatcher) {
            val copies = MutableSharedFlow<PdfCopy>(replay = 1)
            copies.tryEmit(PdfCopy.Failed(PdfCopyFailure.CONNECTION))
            repository.pdfCopyFlow = copies
            val viewModel = viewModel()
            advanceUntilIdle()

            repeat(3) {
                viewModel.retryOpen()
                advanceUntilIdle()
            }

            assertEquals(1, copies.subscriptionCount.value)
            assertEquals(PdfStatus.CONNECTION_FAILED, viewModel.success().pdf?.status)
        }

    @Test
    fun anOldSessionsLateResultNeverTouchesTheNewReader() =
        runTest(testDispatcher) {
            val first = MutableSharedFlow<PdfCopy>()
            repository.pdfCopyFlow = first
            val viewModel = viewModel()
            advanceUntilIdle()
            repository.pdfCopyFlow = noAnswer
            viewModel.retryOpen()
            advanceUntilIdle()

            first.emit(PdfCopy.Ready(PDF_PATH))
            advanceUntilIdle()

            assertEquals(PdfStatus.CONNECTION_FAILED, viewModel.success().pdf?.status)
        }

    @Test
    fun aReplacedOpensLateAnswersNeverOverwriteTheRetry() =
        runTest(testDispatcher) {
            val server = fakeItemDetail(id = "lib_1", itemType = "pdf")
            val first = lastSeen("pdf").copy(libraryEntryId = "lib_1")
            repository.getItemResults = listOf(Result.success(first), Result.success(server))
            val oldAnswers = CompletableDeferred<Unit>()
            repository.highlightsGate = oldAnswers
            repository.metadataGate = oldAnswers
            val viewModel = viewModel()
            advanceUntilIdle()

            repository.highlightsGate = null
            repository.metadataGate = null
            repository.listHighlightsResult = Result.success(listOf(fakeHighlight(id = "hlt_new")))
            repository.getItemNoteResult = Result.success("new note")
            repository.getItemTagsResult = Result.success(listOf("new"))
            viewModel.retryOpen()
            advanceUntilIdle()

            repository.listHighlightsResult = Result.success(listOf(fakeHighlight(id = "hlt_old")))
            repository.getItemNoteResult = Result.success("old note")
            repository.getItemTagsResult = Result.success(listOf("old"))
            oldAnswers.complete(Unit)
            advanceUntilIdle()

            val state = viewModel.success()
            assertEquals(listOf("hlt_new"), state.highlights.map { it.id })
            assertEquals("new note", state.itemNote)
            assertEquals(listOf("new"), state.itemTags)
        }

    @Test
    fun aSecondRetryWhileLoadingSendsNoSecondRequest() =
        runTest(testDispatcher) {
            val viewModel = viewModel()
            advanceUntilIdle()
            val gate = CompletableDeferred<Unit>()
            repository.getItemGate = gate

            viewModel.retryOpen()
            viewModel.retryOpen()
            gate.complete(Unit)
            advanceUntilIdle()

            assertEquals(2, repository.getItemCallCount)
            assertEquals(PdfStatus.CONNECTION_FAILED, viewModel.success().pdf?.status)
        }

    @Test
    fun aRetryWhileTheFirstOpenIsStillRunningReplacesIt() =
        runTest(testDispatcher) {
            val server = fakeItemDetail(id = "lib_1", itemType = "pdf", progressPercent = 60f)
            repository.getItemResults = listOf(Result.success(lastSeen("pdf")), Result.success(server))
            repository.pdfCopyFlow = flowOf(PdfCopy.Ready(PDF_PATH))
            val opened = CompletableDeferred<Unit>()
            events.openedGate = opened
            val viewModel = viewModel()
            advanceUntilIdle()

            viewModel.retryOpen()
            opened.complete(Unit)
            advanceUntilIdle()

            assertEquals(listOf("doc_test1"), events.openedDocuments)
            assertEquals(12, viewModel.success().pdf?.restore?.pageIndex)
        }

    @Test
    fun aReplacedBookStopsLoadingItsChapter() =
        runTest(testDispatcher) {
            val server = fakeItemDetail(itemType = "book").copy(chapterLocator = "e5", chapterOffset = 120)
            repository.getItemResults = listOf(Result.success(lastSeen("book")), Result.success(server))
            repository.epubTocResult = Result.success(bookToc())
            val chapter = CompletableDeferred<Unit>()
            var loaded = 0
            repository.epubChapterResult = { index ->
                chapter.await()
                loaded++
                Result.success("<p>chapter $index</p>")
            }
            val viewModel = viewModel()
            advanceUntilIdle()

            viewModel.retryOpen()
            advanceUntilIdle()
            chapter.complete(Unit)
            advanceUntilIdle()

            assertEquals(1, loaded)
        }

    @Test
    fun lastSeenBookWithNoAnswerShowsTheConnectionScreen() =
        runTest(testDispatcher) {
            repository.getItemResult = Result.success(lastSeen("book"))
            repository.epubTocResult = Result.failure(IOException("offline"))

            val viewModel = viewModel()
            advanceUntilIdle()

            assertEquals(EpubStatus.CONNECTION_FAILED, viewModel.success().epub?.status)
        }

    @Test
    fun retryOnALastSeenBookResumesAtTheServersChapter() =
        runTest(testDispatcher) {
            val server = fakeItemDetail(itemType = "book").copy(chapterLocator = "e5", chapterOffset = 120)
            repository.getItemResults = listOf(Result.success(lastSeen("book")), Result.success(server))
            repository.epubTocResult = Result.failure(IOException("offline"))
            val viewModel = viewModel()
            advanceUntilIdle()
            repository.epubTocResult = Result.success(bookToc())

            viewModel.retryOpen()
            advanceUntilIdle()

            val epub = viewModel.success().epub
            assertEquals("<p>chapter 5</p>", epub?.html)
            assertEquals(120, epub?.restore?.offset)
        }
}
