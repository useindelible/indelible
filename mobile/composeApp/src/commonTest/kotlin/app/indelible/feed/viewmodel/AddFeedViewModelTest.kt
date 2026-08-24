package app.indelible.feed.viewmodel

import app.indelible.core.i18n.UiMessage
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.feed_error_file_too_large
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
import kotlin.test.assertIs

@OptIn(ExperimentalCoroutinesApi::class)
class AddFeedViewModelTest {
    private val testDispatcher = UnconfinedTestDispatcher()
    private lateinit var repository: FakeFeedRepository
    private lateinit var viewModel: AddFeedViewModel

    @BeforeTest
    fun setUp() {
        Dispatchers.setMain(testDispatcher)
        repository = FakeFeedRepository()
        viewModel = AddFeedViewModel(repository)
    }

    @AfterTest
    fun tearDown() {
        Dispatchers.resetMain()
    }

    @Test
    fun import_opml_within_cap_reaches_repository() =
        runTest(testDispatcher) {
            viewModel.importOpml(ByteArray(1024), "subscriptions.opml")
            advanceUntilIdle()

            assertEquals(1, repository.importOpmlCallCount)
            assertEquals("subscriptions.opml", repository.lastImportOpmlFileName)
            assertIs<AddFeedUiState.Idle>(viewModel.uiState.value)
        }

    @Test
    fun import_opml_at_exactly_the_cap_is_accepted() =
        runTest(testDispatcher) {
            // The cap is inclusive everywhere it is enforced (picker and here),
            // so a file of exactly MAX_OPML_BYTES must still import.
            viewModel.importOpml(ByteArray(AddFeedViewModel.MAX_OPML_BYTES.toInt()), "exact.opml")
            advanceUntilIdle()

            assertEquals(1, repository.importOpmlCallCount)
            assertIs<AddFeedUiState.Idle>(viewModel.uiState.value)
        }

    @Test
    fun import_opml_over_cap_errors_without_calling_repository() =
        runTest(testDispatcher) {
            val oversized = ByteArray((AddFeedViewModel.MAX_OPML_BYTES + 1).toInt())

            val effects = mutableListOf<AddFeedEffect>()
            val job = launch { viewModel.effects.toList(effects) }

            viewModel.importOpml(oversized, "huge.opml")
            advanceUntilIdle()

            val state = assertIs<AddFeedUiState.Error>(viewModel.uiState.value)
            assertEquals(Res.string.feed_error_file_too_large, state.message.resource)
            assertEquals(listOf("huge.opml"), state.message.formatArgs)
            assertEquals(0, repository.importOpmlCallCount)
            assertEquals(
                UiMessage(Res.string.feed_error_file_too_large, listOf("huge.opml")),
                effects.filterIsInstance<AddFeedEffect.ShowSnackbar>().single().message,
            )
            job.cancel()
        }

    @Test
    fun picker_rejection_surfaces_the_too_large_error() =
        runTest(testDispatcher) {
            val effects = mutableListOf<AddFeedEffect>()
            val job = launch { viewModel.effects.toList(effects) }

            viewModel.onFileTooLarge("huge.opml")
            advanceUntilIdle()

            val state = assertIs<AddFeedUiState.Error>(viewModel.uiState.value)
            assertEquals(Res.string.feed_error_file_too_large, state.message.resource)
            assertEquals(0, repository.importOpmlCallCount)
            assertEquals(
                UiMessage(Res.string.feed_error_file_too_large, listOf("huge.opml")),
                effects.filterIsInstance<AddFeedEffect.ShowSnackbar>().single().message,
            )
            job.cancel()
        }
}
