package app.indelible.reader.viewmodel

import app.indelible.reader.model.DataPanel
import app.indelible.reader.model.HighlightColor
import app.indelible.reader.viewmodel.FakeReaderRepository.Companion.fakeHighlight
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.advanceTimeBy
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

    @Test
    fun aHighlightWrittenDuringTheRefreshIsKept() =
        runTest(testDispatcher) {
            val viewModel = reader()
            advanceUntilIdle()
            assertIs<ReaderUiState.Success>(viewModel.uiState.value)
            val gate = CompletableDeferred<Unit>()
            repository.highlightsGate = gate

            changes.tryEmit("doc_test1")
            advanceUntilIdle()
            repository.highlightsGate = null
            viewModel.createHighlightForTag(HighlightColor.YELLOW, "kept", 0L, 4L) {}
            advanceUntilIdle()
            gate.complete(Unit)
            advanceUntilIdle()

            val after = assertIs<ReaderUiState.Success>(viewModel.uiState.value)
            assertEquals(listOf(fakeHighlight().id), after.highlights.map { it.id })
        }

    @Test
    fun anOpenNoteSheetKeepsItsNote() =
        runTest(testDispatcher) {
            val viewModel = reader()
            advanceUntilIdle()
            viewModel.openPanel(DataPanel.NOTE)
            repository.getItemNoteResult = Result.success("from the web")

            changes.tryEmit("doc_test1")
            advanceUntilIdle()

            val open = assertIs<ReaderUiState.Success>(viewModel.uiState.value)
            assertEquals(null, open.itemNote)
            viewModel.closePanel()
            changes.tryEmit("doc_test1")
            advanceUntilIdle()
            val closed = assertIs<ReaderUiState.Success>(viewModel.uiState.value)
            assertEquals("from the web", closed.itemNote)
        }

    @Test
    fun openingTheNoteEditorReadsTheServersNote() =
        runTest(testDispatcher) {
            val viewModel = reader()
            advanceUntilIdle()
            val reads = repository.getItemNoteCallCount
            repository.getItemNoteResult = Result.success("from the web")

            viewModel.openPanel(DataPanel.NOTE)
            advanceUntilIdle()

            assertEquals(reads + 1, repository.getItemNoteCallCount)
            assertEquals(true, repository.lastItemNoteFromServer)
            assertEquals("from the web", assertIs<ReaderUiState.Success>(viewModel.uiState.value).itemNote)
        }

    @Test
    fun theNoteEditorWaitsForTheOpenRead() =
        runTest(testDispatcher) {
            val viewModel = reader()
            advanceUntilIdle()
            val gate = CompletableDeferred<Unit>()
            repository.metadataGate = gate

            viewModel.openPanel(DataPanel.NOTE)
            val reading = assertIs<ReaderUiState.Success>(viewModel.uiState.value).noteRefreshing
            gate.complete(Unit)
            advanceUntilIdle()

            assertEquals(true, reading)
            assertEquals(false, assertIs<ReaderUiState.Success>(viewModel.uiState.value).noteRefreshing)
        }

    @Test
    fun aFailedOpenReadKeepsTheNoteAndFreesTheEditor() =
        runTest(testDispatcher) {
            repository.getItemNoteResult = Result.success("loaded")
            val viewModel = reader()
            advanceUntilIdle()
            repository.getItemNoteResult = Result.failure(IllegalStateException("offline"))

            viewModel.openPanel(DataPanel.NOTE)
            advanceUntilIdle()

            val state = assertIs<ReaderUiState.Success>(viewModel.uiState.value)
            assertEquals("loaded", state.itemNote)
            assertEquals(false, state.noteRefreshing)
        }

    @Test
    fun repeatedOpensShareOneRead() =
        runTest(testDispatcher) {
            val viewModel = reader()
            advanceUntilIdle()
            val reads = repository.getItemNoteCallCount
            repository.metadataGate = CompletableDeferred()

            viewModel.openPanel(DataPanel.NOTE)
            viewModel.closePanel()
            viewModel.openPanel(DataPanel.NOTE)

            assertEquals(reads + 1, repository.getItemNoteCallCount)
        }

    @Test
    fun anOpenSheetHoldsTheReconnectNoteRead() =
        runTest(testDispatcher) {
            val viewModel = reader()
            advanceUntilIdle()
            viewModel.openPanel(DataPanel.NOTE)
            advanceUntilIdle()
            val reads = repository.getItemNoteCallCount

            changes.tryEmit("doc_test1")
            advanceUntilIdle()

            assertEquals(reads, repository.getItemNoteCallCount)
        }

    @Test
    fun openingAnotherPanelReadsNothing() =
        runTest(testDispatcher) {
            val viewModel = reader()
            advanceUntilIdle()
            val reads = repository.getItemNoteCallCount

            viewModel.openPanel(DataPanel.AA)
            advanceUntilIdle()

            assertEquals(reads, repository.getItemNoteCallCount)
        }

    @Test
    fun aRefreshWithoutAnAnswerIsTriedAgain() =
        runTest(testDispatcher) {
            val viewModel = reader()
            advanceUntilIdle()
            assertIs<ReaderUiState.Success>(viewModel.uiState.value)
            repository.listHighlightsResult = Result.failure(IllegalStateException("no answer"))
            val before = repository.listHighlightsCallCount

            changes.tryEmit("doc_test1")
            runCurrent()
            assertEquals(before + 1, repository.listHighlightsCallCount)
            repository.listHighlightsResult = Result.success(listOf(fakeHighlight()))
            advanceTimeBy(RECONCILE_FIRST_RETRY_MS)
            runCurrent()

            assertEquals(before + 2, repository.listHighlightsCallCount)
            val after = assertIs<ReaderUiState.Success>(viewModel.uiState.value)
            assertEquals(1, after.highlights.size)
        }

    @Test
    fun aLateFirstNoteReadDoesNotOverwriteTheReconciledNote() =
        runTest(testDispatcher) {
            val firstRead = CompletableDeferred<Unit>()
            repository.metadataGate = firstRead
            val viewModel = reader()
            advanceUntilIdle()
            assertIs<ReaderUiState.Success>(viewModel.uiState.value)
            repository.metadataGate = null
            repository.getItemNoteResult = Result.success("fresh")

            changes.tryEmit("doc_test1")
            advanceUntilIdle()
            repository.getItemNoteResult = Result.success(null)
            firstRead.complete(Unit)
            advanceUntilIdle()

            assertEquals("fresh", assertIs<ReaderUiState.Success>(viewModel.uiState.value).itemNote)
        }

    @Test
    fun aNoteNeverLoadedIsAppliedEvenWithTheSheetOpen() =
        runTest(testDispatcher) {
            repository.getItemNoteResult = Result.failure(IllegalStateException("offline"))
            val viewModel = reader()
            advanceUntilIdle()
            viewModel.openPanel(DataPanel.NOTE)
            repository.getItemNoteResult = Result.success("from the web")

            changes.tryEmit("doc_test1")
            advanceUntilIdle()

            val open = assertIs<ReaderUiState.Success>(viewModel.uiState.value)
            assertEquals("from the web" to true, open.itemNote to open.itemNoteLoaded)
        }

    @Test
    fun aFailedTagReadIsTriedAgain() =
        runTest(testDispatcher) {
            val viewModel = reader()
            advanceUntilIdle()
            repository.getItemTagsResult = Result.failure(IllegalStateException("no answer"))
            val before = repository.getItemTagsCallCount

            changes.tryEmit("doc_test1")
            runCurrent()
            repository.getItemTagsResult = Result.success(listOf("later"))
            advanceTimeBy(RECONCILE_FIRST_RETRY_MS)
            runCurrent()

            assertEquals(before + 2, repository.getItemTagsCallCount)
            assertEquals(listOf("later"), assertIs<ReaderUiState.Success>(viewModel.uiState.value).itemTags)
        }
}
