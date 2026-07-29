package app.indelible.library.viewmodel

import app.indelible.library.viewmodel.FakeLibraryRepository.Companion.fakeLibraryItem
import app.indelible.library.viewmodel.FakeLibraryRepository.Companion.paginatedItems
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
import kotlin.test.assertNull
import kotlin.test.assertTrue

@OptIn(ExperimentalCoroutinesApi::class)
class LibraryViewModelTest {
    private val testDispatcher = UnconfinedTestDispatcher()
    private lateinit var repository: FakeLibraryRepository
    private lateinit var viewModel: LibraryViewModel

    @BeforeTest
    fun setUp() {
        Dispatchers.setMain(testDispatcher)
        repository = FakeLibraryRepository()
    }

    @AfterTest
    fun tearDown() {
        Dispatchers.resetMain()
    }

    @Test
    fun construction_performs_no_request() =
        runTest(testDispatcher) {
            // The container builds this VM before sign-in; an init-time fetch would
            // cache a "Session expired" error that the library tab then shows post-login.
            viewModel = LibraryViewModel(repository)
            advanceUntilIdle()

            assertEquals(0, repository.listItemsCallCount)
        }

    @Test
    fun load_items_success() =
        runTest(testDispatcher) {
            val items =
                listOf(
                    fakeLibraryItem("a"),
                    fakeLibraryItem("b"),
                    fakeLibraryItem("c"),
                )
            repository.listItemsResult = Result.success(paginatedItems(items))

            viewModel = LibraryViewModel(repository)
            viewModel.refresh()
            advanceUntilIdle()

            val state = assertIs<LibraryUiState.Success>(viewModel.uiState.value)
            assertEquals(3, state.items.size)
        }

    @Test
    fun load_items_error() =
        runTest(testDispatcher) {
            repository.listItemsResult = Result.failure(RuntimeException("Network error"))

            viewModel = LibraryViewModel(repository)
            viewModel.refresh()
            advanceUntilIdle()

            val state = assertIs<LibraryUiState.Error>(viewModel.uiState.value)
            assertEquals("Network error", state.message)
        }

    @Test
    fun triage_item_optimistic_remove() =
        runTest(testDispatcher) {
            val item = fakeLibraryItem("target")
            repository.listItemsResult = Result.success(paginatedItems(listOf(item, fakeLibraryItem("other"))))

            viewModel = LibraryViewModel(repository)
            viewModel.refresh()
            advanceUntilIdle()

            viewModel.triageItem(item, "later")

            val state = assertIs<LibraryUiState.Success>(viewModel.uiState.value)
            assertTrue(state.items.none { it.id == "target" })
            assertEquals(1, state.items.size)
        }

    @Test
    fun triage_item_error_restores_item() =
        runTest(testDispatcher) {
            val item = fakeLibraryItem("target")
            repository.listItemsResult = Result.success(paginatedItems(listOf(item, fakeLibraryItem("other"))))
            repository.triageItemResult = Result.failure(RuntimeException("Server error"))

            viewModel = LibraryViewModel(repository)
            viewModel.refresh()
            advanceUntilIdle()

            viewModel.triageItem(item, "later")
            advanceUntilIdle()

            val state = assertIs<LibraryUiState.Success>(viewModel.uiState.value)
            assertTrue(state.items.any { it.id == "target" })
            assertEquals(2, state.items.size)
        }

    @Test
    fun load_next_page_appends() =
        runTest(testDispatcher) {
            val page1 = listOf(fakeLibraryItem("a"), fakeLibraryItem("b"))
            val page2 = listOf(fakeLibraryItem("c"), fakeLibraryItem("d"))

            repository.listItemsResult =
                Result.success(
                    paginatedItems(page1, hasMore = true, nextCursor = "cursor1"),
                )

            viewModel = LibraryViewModel(repository)
            viewModel.refresh()
            advanceUntilIdle()

            repository.listItemsResult = Result.success(paginatedItems(page2, hasMore = false))
            viewModel.loadNextPage()
            advanceUntilIdle()

            val state = assertIs<LibraryUiState.Success>(viewModel.uiState.value)
            assertEquals(4, state.items.size)
            assertEquals("cursor1", repository.lastListItemsCursor)
        }

    @Test
    fun filter_by_triage_state() =
        runTest(testDispatcher) {
            repository.listItemsResult = Result.success(FakeLibraryRepository.emptyPaginatedItems())
            viewModel = LibraryViewModel(repository)
            viewModel.refresh()
            advanceUntilIdle()

            viewModel.setTriageFilter(TriageFilter.LATER)
            advanceUntilIdle()

            assertEquals("later", repository.lastListItemsTriageState)
        }

    @Test
    fun filter_by_content_type() =
        runTest(testDispatcher) {
            repository.listItemsResult = Result.success(FakeLibraryRepository.emptyPaginatedItems())
            viewModel = LibraryViewModel(repository)
            viewModel.refresh()
            advanceUntilIdle()

            viewModel.setContentTypeFilter(ContentTypeFilter.ARTICLES)
            advanceUntilIdle()

            assertEquals("article", repository.lastListItemsItemType)
        }

    @Test
    fun initial_triage_filter_is_inbox() =
        runTest(testDispatcher) {
            repository.listItemsResult = Result.success(FakeLibraryRepository.emptyPaginatedItems())
            viewModel = LibraryViewModel(repository)
            viewModel.refresh()
            advanceUntilIdle()

            assertEquals("inbox", repository.lastListItemsTriageState)
            assertNull(repository.lastListItemsItemType)
        }
}
