package app.indelible.reader.viewmodel

import app.indelible.reader.model.HighlightColor
import app.indelible.reader.viewmodel.FakeReaderRepository.Companion.fakeHighlight
import app.indelible.reader.viewmodel.FakeReaderRepository.Companion.fakeItemDetail
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
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
import kotlin.test.assertNotNull
import kotlin.test.assertNull
import kotlin.test.assertTrue

@OptIn(ExperimentalCoroutinesApi::class)
class ReaderHighlightStateTest {
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
    fun create_highlight_adds_to_state() =
        runTest(testDispatcher) {
            val highlight = fakeHighlight(id = "hlt_new", color = "Green")
            repository.getItemResult = Result.success(fakeItemDetail())
            repository.createHighlightResult = Result.success(highlight)
            viewModel = ReaderViewModel("doc_test1", repository, readingEvents = events)
            advanceUntilIdle()

            viewModel.createHighlight(HighlightColor.GREEN, "text", 0, 10)
            advanceUntilIdle()

            val state = assertIs<ReaderUiState.Success>(viewModel.uiState.value)
            assertTrue(state.highlights.any { it.id == "hlt_new" })
            assertEquals(1, repository.createHighlightCallCount)
        }

    @Test
    fun delete_highlight_removes_from_state() =
        runTest(testDispatcher) {
            val highlight = fakeHighlight(id = "hlt_remove")
            repository.getItemResult = Result.success(fakeItemDetail())
            repository.listHighlightsResult = Result.success(listOf(highlight))
            viewModel = ReaderViewModel("doc_test1", repository, readingEvents = events)
            advanceUntilIdle()

            viewModel.deleteHighlight("hlt_remove")
            advanceUntilIdle()

            val state = assertIs<ReaderUiState.Success>(viewModel.uiState.value)
            assertTrue(state.highlights.none { it.id == "hlt_remove" })
            assertEquals("hlt_remove", repository.lastDeletedHighlightId)
        }

    @Test
    fun delete_highlight_rollback_on_failure() =
        runTest(testDispatcher) {
            val highlight = fakeHighlight(id = "hlt_fail")
            repository.getItemResult = Result.success(fakeItemDetail())
            repository.listHighlightsResult = Result.success(listOf(highlight))
            repository.deleteHighlightResult = Result.failure(RuntimeException("Server error"))
            viewModel = ReaderViewModel("doc_test1", repository, readingEvents = events)
            advanceUntilIdle()

            viewModel.deleteHighlight("hlt_fail")
            advanceUntilIdle()

            val state = assertIs<ReaderUiState.Success>(viewModel.uiState.value)
            assertTrue(state.highlights.any { it.id == "hlt_fail" })
        }

    @Test
    fun update_highlight_color_updates_state() =
        runTest(testDispatcher) {
            val highlight = fakeHighlight(id = "hlt_color", color = "Yellow")
            val updated = highlight.copy(color = "Blue")
            repository.getItemResult = Result.success(fakeItemDetail())
            repository.listHighlightsResult = Result.success(listOf(highlight))
            repository.updateHighlightColorResult = Result.success(updated)
            viewModel = ReaderViewModel("doc_test1", repository, readingEvents = events)
            advanceUntilIdle()

            viewModel.updateHighlightColor("hlt_color", HighlightColor.BLUE)
            advanceUntilIdle()

            val state = assertIs<ReaderUiState.Success>(viewModel.uiState.value)
            val found = state.highlights.find { it.id == "hlt_color" }
            assertNotNull(found)
            assertEquals("Blue", found.color)
        }

    @Test
    fun highlights_loaded_on_init() =
        runTest(testDispatcher) {
            val highlights =
                listOf(
                    fakeHighlight(id = "hlt_1"),
                    fakeHighlight(id = "hlt_2"),
                )
            repository.getItemResult = Result.success(fakeItemDetail())
            repository.listHighlightsResult = Result.success(highlights)
            viewModel = ReaderViewModel("doc_test1", repository, readingEvents = events)
            advanceUntilIdle()

            val state = assertIs<ReaderUiState.Success>(viewModel.uiState.value)
            assertEquals(2, state.highlights.size)
        }

    @Test
    fun upsert_note_updates_highlight_in_state() =
        runTest(testDispatcher) {
            val highlight = fakeHighlight(id = "hlt_note")
            val note = FakeReaderRepository.fakeNote(highlightId = "hlt_note")
            repository.getItemResult = Result.success(fakeItemDetail())
            repository.listHighlightsResult = Result.success(listOf(highlight))
            repository.upsertNoteResult = Result.success(note)
            viewModel = ReaderViewModel("doc_test1", repository, readingEvents = events)
            advanceUntilIdle()

            assertNull(
                (viewModel.uiState.value as ReaderUiState.Success)
                    .highlights
                    .first()
                    .note,
            )

            viewModel.upsertHighlightNote("hlt_note", "my note")
            advanceUntilIdle()

            val state = assertIs<ReaderUiState.Success>(viewModel.uiState.value)
            val found = state.highlights.find { it.id == "hlt_note" }
            assertNotNull(found?.note)
            assertEquals("test note", found.note?.body)
        }
}
