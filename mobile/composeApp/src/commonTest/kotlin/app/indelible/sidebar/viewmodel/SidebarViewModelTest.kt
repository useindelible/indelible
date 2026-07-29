package app.indelible.sidebar.viewmodel

import app.indelible.sidebar.viewmodel.FakeSidebarRepository.Companion.collection
import app.indelible.sidebar.viewmodel.FakeSidebarRepository.Companion.smartList
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
import kotlin.test.assertTrue

@OptIn(ExperimentalCoroutinesApi::class)
class SidebarViewModelTest {
    private val testDispatcher = UnconfinedTestDispatcher()
    private lateinit var repository: FakeSidebarRepository

    @BeforeTest
    fun setUp() {
        Dispatchers.setMain(testDispatcher)
        repository = FakeSidebarRepository()
    }

    @AfterTest
    fun tearDown() {
        Dispatchers.resetMain()
    }

    @Test
    fun loads_collections_and_smart_lists() =
        runTest(testDispatcher) {
            repository.collectionsResult = Result.success(listOf(collection("col_1", "Reading")))
            repository.smartListsResult = Result.success(listOf(smartList("sl_1", "Unread")))

            val viewModel = SidebarViewModel(repository)
            viewModel.load()
            advanceUntilIdle()

            val state = assertIs<SidebarUiState.Ready>(viewModel.uiState.value)
            assertEquals(1, state.collections.size)
            assertEquals(1, state.smartLists.size)
            assertEquals("Reading", state.collections.first().name)
            assertEquals("Unread", state.smartLists.first().name)
        }

    @Test
    fun empty_results_still_reach_ready_state() =
        runTest(testDispatcher) {
            val viewModel = SidebarViewModel(repository)
            viewModel.load()
            advanceUntilIdle()

            val state = assertIs<SidebarUiState.Ready>(viewModel.uiState.value)
            assertTrue(state.collections.isEmpty())
            assertTrue(state.smartLists.isEmpty())
        }

    @Test
    fun collection_failure_degrades_to_empty_without_dropping_smart_lists() =
        runTest(testDispatcher) {
            repository.collectionsResult = Result.failure(RuntimeException("offline"))
            repository.smartListsResult = Result.success(listOf(smartList("sl_1", "Unread")))

            val viewModel = SidebarViewModel(repository)
            viewModel.load()
            advanceUntilIdle()

            val state = assertIs<SidebarUiState.Ready>(viewModel.uiState.value)
            assertTrue(state.collections.isEmpty())
            assertEquals(1, state.smartLists.size)
        }
}
