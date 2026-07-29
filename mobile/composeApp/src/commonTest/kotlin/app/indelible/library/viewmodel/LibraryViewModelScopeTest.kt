package app.indelible.library.viewmodel

import app.indelible.core.model.LibraryCounts
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
import kotlin.test.assertNotNull
import kotlin.test.assertNull
import kotlin.test.assertTrue

@OptIn(ExperimentalCoroutinesApi::class)
class LibraryViewModelScopeTest {
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
    fun collection_scope_loads_collection_items() =
        runTest(testDispatcher) {
            repository.listCollectionItemsResult =
                Result.success(paginatedItems(listOf(fakeLibraryItem("a"), fakeLibraryItem("b"))))
            viewModel = LibraryViewModel(repository)
            advanceUntilIdle()

            viewModel.setScope(LibraryScope.Collection("col_1", "Reading"))
            advanceUntilIdle()

            assertEquals("col_1", repository.lastCollectionItemsId)
            assertEquals(LibraryScope.Collection("col_1", "Reading"), viewModel.scope.value)
            val state = assertIs<LibraryUiState.Success>(viewModel.uiState.value)
            assertEquals(2, state.items.size)
        }

    @Test
    fun smart_list_scope_loads_smart_list_items() =
        runTest(testDispatcher) {
            repository.listSmartListItemsResult =
                Result.success(paginatedItems(listOf(fakeLibraryItem("a"))))
            viewModel = LibraryViewModel(repository)
            advanceUntilIdle()

            viewModel.setScope(LibraryScope.SmartList("sl_1", "Unread"))
            advanceUntilIdle()

            assertEquals("sl_1", repository.lastSmartListItemsId)
            assertEquals(LibraryScope.SmartList("sl_1", "Unread"), viewModel.scope.value)
        }

    @Test
    fun triage_filter_resets_scope_to_triage() =
        runTest(testDispatcher) {
            repository.listCollectionItemsResult = Result.success(FakeLibraryRepository.emptyPaginatedItems())
            viewModel = LibraryViewModel(repository)
            advanceUntilIdle()

            viewModel.setScope(LibraryScope.Collection("col_1", "Reading"))
            advanceUntilIdle()

            viewModel.setTriageFilter(TriageFilter.LATER)
            advanceUntilIdle()

            assertTrue(viewModel.scope.value is LibraryScope.Triage)
            assertEquals("later", repository.lastListItemsTriageState)
        }

    @Test
    fun collection_scope_paginates_with_cursor() =
        runTest(testDispatcher) {
            repository.listCollectionItemsResult =
                Result.success(
                    paginatedItems(
                        listOf(fakeLibraryItem("a"), fakeLibraryItem("b")),
                        hasMore = true,
                        nextCursor = "cursor1",
                    ),
                )
            viewModel = LibraryViewModel(repository)
            advanceUntilIdle()

            viewModel.setScope(LibraryScope.Collection("col_1", "Reading"))
            advanceUntilIdle()

            repository.listCollectionItemsResult =
                Result.success(paginatedItems(listOf(fakeLibraryItem("c")), hasMore = false))
            viewModel.loadNextPage()
            advanceUntilIdle()

            assertEquals("cursor1", repository.lastCollectionItemsCursor)
            val state = assertIs<LibraryUiState.Success>(viewModel.uiState.value)
            assertEquals(3, state.items.size)
        }

    @Test
    fun triage_scope_loads_counts_for_the_selected_triage_state() =
        runTest(testDispatcher) {
            repository.scopeCountsResult =
                Result.success(
                    LibraryCounts(
                        total = 9,
                        unread = 4,
                        reading = 3,
                        done = 2,
                        byItemType = mapOf("article" to 7, "video" to 2),
                    ),
                )
            viewModel = LibraryViewModel(repository)
            advanceUntilIdle()

            viewModel.setTriageFilter(TriageFilter.LATER)
            advanceUntilIdle()

            assertEquals("later", repository.lastScopeCountsTriageState)
            val counts = assertNotNull(viewModel.counts.value)
            assertEquals(9, counts.total)
            assertEquals(7, counts.byItemType["article"])
        }

    @Test
    fun non_triage_scope_clears_counts_without_calling_the_endpoint() =
        runTest(testDispatcher) {
            repository.scopeCountsResult = Result.success(LibraryCounts(1, 1, 0, 0, emptyMap()))
            viewModel = LibraryViewModel(repository)
            viewModel.setTriageFilter(TriageFilter.INBOX)
            advanceUntilIdle()
            assertNotNull(viewModel.counts.value)
            val callsAfterTriage = repository.scopeCountsCallCount

            viewModel.setScope(LibraryScope.Collection("col_1", "Reading"))
            advanceUntilIdle()

            assertNull(viewModel.counts.value)
            assertEquals(callsAfterTriage, repository.scopeCountsCallCount)
        }

    @Test
    fun counts_failure_leaves_the_list_intact() =
        runTest(testDispatcher) {
            repository.listItemsResult = Result.success(paginatedItems(listOf(fakeLibraryItem("a"))))
            repository.scopeCountsResult = Result.failure(RuntimeException("boom"))
            viewModel = LibraryViewModel(repository)

            viewModel.refresh()
            advanceUntilIdle()

            assertNull(viewModel.counts.value)
            val state = assertIs<LibraryUiState.Success>(viewModel.uiState.value)
            assertEquals(1, state.items.size)
        }
}
