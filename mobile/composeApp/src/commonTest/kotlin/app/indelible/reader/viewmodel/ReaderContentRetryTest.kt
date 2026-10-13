package app.indelible.reader.viewmodel

import app.indelible.core.i18n.UiMessage
import app.indelible.reader.model.ReaderReprocessResult
import app.indelible.reader.viewmodel.FakeReaderRepository.Companion.fakeItemDetail
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.reader_error_retry_content
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
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
import kotlin.test.assertNull
import kotlin.test.assertTrue

@OptIn(ExperimentalCoroutinesApi::class)
class ReaderContentRetryTest {
    private val testDispatcher = UnconfinedTestDispatcher()
    private lateinit var repository: FakeReaderRepository
    private lateinit var events: RecordingReadingEventWriter
    private lateinit var viewModel: ReaderViewModel

    @BeforeTest
    fun setUp() {
        Dispatchers.setMain(testDispatcher)
        repository = FakeReaderRepository()
        events = RecordingReadingEventWriter()
    }

    @AfterTest
    fun tearDown() {
        Dispatchers.resetMain()
    }

    @Test
    fun polls_until_readable_then_loads_html() =
        runTest(testDispatcher) {
            repository.getItemResults =
                listOf(
                    Result.success(fakeItemDetail(readableReady = false)),
                    Result.success(fakeItemDetail(readableReady = true)),
                )
            repository.fetchHtmlResult = Result.success("<p>Prepared content</p>")
            viewModel = ReaderViewModel("doc_test1", repository, readingEvents = events)
            advanceUntilIdle()

            val state = assertIs<ReaderUiState.Success>(viewModel.uiState.value)
            assertEquals(ReaderContentStatus.READY, state.contentStatus)
            assertEquals("<p>Prepared content</p>", state.htmlContent)
        }

    @Test
    fun content_unavailable_after_polls_exhausted() =
        runTest(testDispatcher) {
            repository.getItemResult = Result.success(fakeItemDetail(readableReady = false))
            viewModel = ReaderViewModel("doc_test1", repository, readingEvents = events)
            advanceUntilIdle()

            val state = assertIs<ReaderUiState.Success>(viewModel.uiState.value)
            assertEquals(ReaderContentStatus.UNAVAILABLE, state.contentStatus)
            assertNull(state.htmlContent)
        }

    @Test
    fun retry_reloads_content_after_unavailable() =
        runTest(testDispatcher) {
            repository.getItemResult = Result.success(fakeItemDetail(readableReady = false))
            viewModel = ReaderViewModel("doc_test1", repository, readingEvents = events)
            advanceUntilIdle()
            assertEquals(
                ReaderContentStatus.UNAVAILABLE,
                assertIs<ReaderUiState.Success>(viewModel.uiState.value).contentStatus,
            )

            repository.getItemResult = Result.success(fakeItemDetail(readableReady = true))
            repository.fetchHtmlResult = Result.success("<p>Now ready</p>")
            viewModel.retryLoadContent()
            advanceUntilIdle()

            val state = assertIs<ReaderUiState.Success>(viewModel.uiState.value)
            assertEquals(ReaderContentStatus.READY, state.contentStatus)
            assertEquals("<p>Now ready</p>", state.htmlContent)
        }

    @Test
    fun retry_reprocesses_document_before_polling_or_fetching_content() =
        runTest(testDispatcher) {
            repository.getItemResult = Result.success(fakeItemDetail(readableReady = false))
            viewModel = ReaderViewModel("doc_test1", repository, readingEvents = events)
            advanceUntilIdle()
            assertEquals(
                ReaderContentStatus.UNAVAILABLE,
                assertIs<ReaderUiState.Success>(viewModel.uiState.value).contentStatus,
            )

            repository.contentCallLog.clear()
            repository.getItemResult = Result.success(fakeItemDetail(readableReady = true))
            repository.fetchHtmlResult = Result.success("<p>Now ready</p>")

            viewModel.retryLoadContent()
            advanceUntilIdle()

            val reprocessIndex = repository.contentCallLog.indexOf("reprocessDocument:doc_test1")
            val pollIndex = repository.contentCallLog.indexOf("getItem:doc_test1")
            val fetchIndex = repository.contentCallLog.indexOf("fetchReadableHtml:doc_test1")
            assertTrue(reprocessIndex >= 0, repository.contentCallLog.toString())
            assertTrue(pollIndex > reprocessIndex, repository.contentCallLog.toString())
            assertTrue(fetchIndex > reprocessIndex, repository.contentCallLog.toString())
        }

    @Test
    fun failed_retry_reprocess_keeps_content_unavailable_and_emits_error() =
        runTest(testDispatcher) {
            repository.getItemResult = Result.success(fakeItemDetail(readableReady = false))
            viewModel = ReaderViewModel("doc_test1", repository, readingEvents = events)
            advanceUntilIdle()
            assertEquals(
                ReaderContentStatus.UNAVAILABLE,
                assertIs<ReaderUiState.Success>(viewModel.uiState.value).contentStatus,
            )

            val effects = mutableListOf<ReaderEffect>()
            val job = launch { viewModel.effects.toList(effects) }
            repository.contentCallLog.clear()
            repository.reprocessDocumentResult = Result.failure(RuntimeException("Queue failed"))
            repository.getItemResult = Result.success(fakeItemDetail(readableReady = true))
            repository.fetchHtmlResult = Result.success("<p>Should not load</p>")

            viewModel.retryLoadContent()
            advanceUntilIdle()
            job.cancel()

            val state = assertIs<ReaderUiState.Success>(viewModel.uiState.value)
            assertEquals(ReaderContentStatus.UNAVAILABLE, state.contentStatus)
            assertNull(state.htmlContent)
            assertTrue(
                effects
                    .filterIsInstance<ReaderEffect.ShowSnackbar>()
                    .any { it.message == UiMessage(Res.string.reader_error_retry_content) },
                effects.toString(),
            )
            assertEquals(listOf("reprocessDocument:doc_test1"), repository.contentCallLog)
        }

    @Test
    fun retry_surfaces_server_cooldown_while_polling() =
        runTest(testDispatcher) {
            repository.getItemResult = Result.success(fakeItemDetail(readableReady = false))
            viewModel = ReaderViewModel("doc_test1", repository, readingEvents = events)
            advanceUntilIdle()
            repository.reprocessDocumentResult =
                Result.success(ReaderReprocessResult(queued = false, retryAfterSeconds = 45))

            viewModel.retryLoadContent()
            runCurrent()

            val cooling = assertIs<ReaderUiState.Success>(viewModel.uiState.value)
            assertEquals(ReaderRetryStatus.COOLDOWN, cooling.retryStatus)
            assertEquals(45L, cooling.retryAfterSeconds)

            advanceUntilIdle()
            val exhausted = assertIs<ReaderUiState.Success>(viewModel.uiState.value)
            assertEquals(ReaderRetryStatus.IDLE, exhausted.retryStatus)
            assertEquals(ReaderContentStatus.UNAVAILABLE, exhausted.contentStatus)
        }
}
