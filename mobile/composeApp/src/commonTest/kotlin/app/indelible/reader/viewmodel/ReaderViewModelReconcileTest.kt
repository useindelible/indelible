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

@OptIn(ExperimentalCoroutinesApi::class)
class ReaderViewModelReconcileTest {
    private val testDispatcher = UnconfinedTestDispatcher()
    private val repository = FakeReaderRepository()
    private val events = RecordingReadingEventWriter()
    private val online = MutableStateFlow(true)
    private val reachable = MutableStateFlow<Boolean?>(true)
    private val changes = MutableSharedFlow<String>(extraBufferCapacity = 16)

    @BeforeTest
    fun setUp() = Dispatchers.setMain(testDispatcher)

    @AfterTest
    fun tearDown() = Dispatchers.resetMain()

    private fun reader() =
        ReaderViewModel(
            "doc_test1",
            repository,
            readingEvents = events,
            triggers = ReaderTriggers(online, reachable, changes),
        )

    @Test
    fun reconnectRefetchesHighlightsAndNoteWithoutReloadingTheDocument() =
        runTest(testDispatcher) {
            val viewModel = reader()
            advanceUntilIdle()
            val opened = assertIs<ReaderUiState.Success>(viewModel.uiState.value)
            val items = repository.getItemCallCount
            val highlights = repository.listHighlightsCallCount
            val notes = repository.getItemNoteCallCount

            online.value = false
            advanceUntilIdle()
            online.value = true
            advanceUntilIdle()

            assertEquals(highlights + 1, repository.listHighlightsCallCount)
            assertEquals(notes + 1, repository.getItemNoteCallCount)
            assertEquals(items, repository.getItemCallCount)
            val after = assertIs<ReaderUiState.Success>(viewModel.uiState.value)
            assertEquals(opened.progress, after.progress)
        }

    @Test
    fun aResolutionWhileOnTheBackStackUpdatesTheReaderWithoutMovingIt() =
        runTest(testDispatcher) {
            val viewModel = reader()
            advanceUntilIdle()
            val opened = assertIs<ReaderUiState.Success>(viewModel.uiState.value)
            assertEquals(emptyList(), opened.highlights)
            repository.listHighlightsResult = Result.success(listOf(fakeHighlight()))
            repository.getItemNoteResult = Result.success("kept from the web")

            changes.tryEmit("doc_test1")
            advanceUntilIdle()

            val after = assertIs<ReaderUiState.Success>(viewModel.uiState.value)
            assertEquals(1, after.highlights.size)
            assertEquals("kept from the web", after.itemNote)
            assertEquals(opened.progress, after.progress)
        }

    @Test
    fun reconnectBeforeSuccessDoesNothing() =
        runTest(testDispatcher) {
            repository.getItemGate = CompletableDeferred()
            val viewModel = reader()
            advanceUntilIdle()
            assertIs<ReaderUiState.Loading>(viewModel.uiState.value)

            online.value = false
            advanceUntilIdle()
            online.value = true
            advanceUntilIdle()

            assertEquals(0, repository.listHighlightsCallCount)
            assertEquals(0, repository.getItemNoteCallCount)
            repository.getItemGate?.complete(Unit)
        }
}
