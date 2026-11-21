package app.indelible.reader.viewmodel

import app.indelible.reader.viewmodel.FakeReaderRepository.Companion.fakeHighlight
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
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

/** The reads a reader opens with can answer after a reconnect refresh; the older answer must not win. */
@OptIn(ExperimentalCoroutinesApi::class)
class ReaderViewModelLateReadTest {
    private val testDispatcher = UnconfinedTestDispatcher()
    private val repository = FakeReaderRepository()
    private val changes = MutableSharedFlow<String>(extraBufferCapacity = 16)

    @BeforeTest
    fun setUp() = Dispatchers.setMain(testDispatcher)

    @AfterTest
    fun tearDown() = Dispatchers.resetMain()

    private fun reader() =
        ReaderViewModel(
            "doc_test1",
            repository,
            readingEvents = RecordingReadingEventWriter(),
            triggers = ReaderTriggers(MutableStateFlow(true), MutableStateFlow(true), changes),
        )

    @Test
    fun aLateFirstHighlightReadDoesNotOverwriteTheReconciledHighlights() =
        runTest(testDispatcher) {
            val firstRead = CompletableDeferred<Unit>()
            repository.highlightsGate = firstRead
            val viewModel = reader()
            advanceUntilIdle()
            assertIs<ReaderUiState.Success>(viewModel.uiState.value)
            repository.highlightsGate = null
            val fresh = fakeHighlight(id = "hlt_fresh")
            repository.listHighlightsResult = Result.success(listOf(fresh))

            changes.tryEmit("doc_test1")
            advanceUntilIdle()
            repository.listHighlightsResult = Result.success(emptyList())
            firstRead.complete(Unit)
            advanceUntilIdle()

            assertEquals(listOf(fresh), assertIs<ReaderUiState.Success>(viewModel.uiState.value).highlights)
        }

    @Test
    fun aLateFirstTagReadDoesNotOverwriteTheReconciledTags() =
        runTest(testDispatcher) {
            val firstRead = CompletableDeferred<Unit>()
            repository.metadataGate = firstRead
            val viewModel = reader()
            advanceUntilIdle()
            assertIs<ReaderUiState.Success>(viewModel.uiState.value)
            repository.metadataGate = null
            repository.getItemTagsResult = Result.success(listOf("fresh"))

            changes.tryEmit("doc_test1")
            advanceUntilIdle()
            repository.getItemTagsResult = Result.success(emptyList())
            firstRead.complete(Unit)
            advanceUntilIdle()

            assertEquals(listOf("fresh"), assertIs<ReaderUiState.Success>(viewModel.uiState.value).itemTags)
        }
}
