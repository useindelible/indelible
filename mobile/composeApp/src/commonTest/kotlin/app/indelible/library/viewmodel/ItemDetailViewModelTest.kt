package app.indelible.library.viewmodel

import app.indelible.core.i18n.UiMessage
import app.indelible.library.viewmodel.FakeLibraryRepository.Companion.fakeItemDetail
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.library_error_load_item
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.toList
import kotlinx.coroutines.launch
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import kotlin.test.AfterTest
import kotlin.test.BeforeTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertIs
import kotlin.test.assertTrue

@OptIn(ExperimentalCoroutinesApi::class)
class ItemDetailViewModelTest {
    private val testDispatcher = UnconfinedTestDispatcher()
    private lateinit var repository: FakeLibraryRepository
    private lateinit var viewModel: ItemDetailViewModel

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
    fun load_item_success() =
        runTest(testDispatcher) {
            repository.getItemResult = Result.success(fakeItemDetail("item42"))
            viewModel = ItemDetailViewModel("item42", repository)
            advanceUntilIdle()

            val state = assertIs<ItemDetailUiState.Success>(viewModel.uiState.value)
            assertEquals("item42", state.item.id)
        }

    @Test
    fun load_item_error() =
        runTest(testDispatcher) {
            repository.getItemResult = Result.failure(RuntimeException("Not found"))
            viewModel = ItemDetailViewModel("item42", repository)
            advanceUntilIdle()

            val state = assertIs<ItemDetailUiState.Error>(viewModel.uiState.value)
            assertEquals(UiMessage(Res.string.library_error_load_item), state.message)
        }

    @Test
    fun favorite_toggle_optimistic() =
        runTest(testDispatcher) {
            repository.getItemResult = Result.success(fakeItemDetail("item1", isFavorite = false))
            viewModel = ItemDetailViewModel("item1", repository)
            advanceUntilIdle()

            repository.toggleFavoriteResult = Result.success(fakeItemDetail("item1", isFavorite = true))
            viewModel.toggleFavorite()

            val state = assertIs<ItemDetailUiState.Success>(viewModel.uiState.value)
            assertTrue(state.item.isFavorite, "Expected optimistic favorite flip")
        }

    @Test
    fun favorite_toggle_rollback_on_error() =
        runTest(testDispatcher) {
            repository.getItemResult = Result.success(fakeItemDetail("item1", isFavorite = false))
            viewModel = ItemDetailViewModel("item1", repository)
            advanceUntilIdle()

            repository.toggleFavoriteResult = Result.failure(RuntimeException("Server error"))
            viewModel.toggleFavorite()
            advanceUntilIdle()

            val state = assertIs<ItemDetailUiState.Success>(viewModel.uiState.value)
            assertFalse(state.item.isFavorite, "Expected rollback after error")
        }

    @Test
    fun delete_item_emits_navigate_back() =
        runTest(testDispatcher) {
            repository.getItemResult = Result.success(fakeItemDetail())
            repository.deleteItemResult = Result.success(Unit)
            viewModel = ItemDetailViewModel("item1", repository)
            advanceUntilIdle()

            val collectedEffects = mutableListOf<ItemDetailEffect>()
            val job = launch { viewModel.effects.toList(collectedEffects) }

            viewModel.deleteItem()
            advanceUntilIdle()

            job.cancel()

            assertTrue(collectedEffects.any { it is ItemDetailEffect.NavigateBack })
        }
}
