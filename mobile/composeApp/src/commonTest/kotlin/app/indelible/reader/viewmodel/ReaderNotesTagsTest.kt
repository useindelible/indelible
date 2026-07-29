package app.indelible.reader.viewmodel

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

@OptIn(ExperimentalCoroutinesApi::class)
class ReaderNotesTagsTest {
    private val testDispatcher = UnconfinedTestDispatcher()
    private lateinit var repository: FakeReaderRepository

    @BeforeTest
    fun setUp() {
        Dispatchers.setMain(testDispatcher)
        repository = FakeReaderRepository()
    }

    @AfterTest
    fun tearDown() {
        Dispatchers.resetMain()
    }

    @Test
    fun item_note_and_tags_load_into_success_state() =
        runTest(testDispatcher) {
            repository.getItemResult = Result.success(fakeItemDetail())
            repository.getItemNoteResult = Result.success("My thoughts on this piece")
            repository.getItemTagsResult = Result.success(listOf("research", "ai"))
            val viewModel = ReaderViewModel("doc_test1", repository)
            advanceUntilIdle()

            val state = assertIs<ReaderUiState.Success>(viewModel.uiState.value)
            assertEquals("My thoughts on this piece", state.itemNote)
            assertEquals(listOf("research", "ai"), state.itemTags)
        }

    @Test
    fun save_item_note_calls_upsert_and_updates_state() =
        runTest(testDispatcher) {
            repository.getItemResult = Result.success(fakeItemDetail())
            val viewModel = ReaderViewModel("doc_test1", repository)
            advanceUntilIdle()

            viewModel.saveItemNote("a fresh note")
            advanceUntilIdle()

            assertEquals("a fresh note", repository.lastUpsertedNote)
            val state = assertIs<ReaderUiState.Success>(viewModel.uiState.value)
            assertEquals("a fresh note", state.itemNote)
        }

    @Test
    fun set_item_tags_round_trips_through_repository() =
        runTest(testDispatcher) {
            repository.getItemResult = Result.success(fakeItemDetail())
            repository.setItemTagsResult = Result.success(listOf("kotlin", "compose"))
            val viewModel = ReaderViewModel("doc_test1", repository)
            advanceUntilIdle()

            viewModel.setItemTags(listOf("kotlin", "compose"))
            advanceUntilIdle()

            assertEquals(listOf("kotlin", "compose"), repository.lastSetItemTags)
            val state = assertIs<ReaderUiState.Success>(viewModel.uiState.value)
            assertEquals(listOf("kotlin", "compose"), state.itemTags)
        }

    @Test
    fun tag_calls_use_library_entry_id_not_route_arg() =
        runTest(testDispatcher) {
            repository.getItemResult =
                Result.success(fakeItemDetail(id = "lib_route", documentId = "doc_route"))
            val viewModel = ReaderViewModel("doc_route", repository)
            advanceUntilIdle()

            assertEquals("lib_route", repository.lastGetItemTagsId)

            viewModel.setItemTags(listOf("kotlin"))
            advanceUntilIdle()

            assertEquals("lib_route", repository.lastSetItemTagsId)
        }
}
