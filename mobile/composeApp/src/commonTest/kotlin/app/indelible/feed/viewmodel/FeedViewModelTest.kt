package app.indelible.feed.viewmodel

import app.indelible.feed.viewmodel.FakeFeedRepository.Companion.fakeFeedItem
import app.indelible.feed.viewmodel.FakeFeedRepository.Companion.fakeSubscription
import app.indelible.feed.viewmodel.FakeFeedRepository.Companion.paginatedFeedItems
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
class FeedViewModelTest {
    private val testDispatcher = UnconfinedTestDispatcher()
    private lateinit var repository: FakeFeedRepository
    private lateinit var viewModel: FeedViewModel

    @BeforeTest
    fun setUp() {
        Dispatchers.setMain(testDispatcher)
        repository = FakeFeedRepository()
    }

    @AfterTest
    fun tearDown() {
        Dispatchers.resetMain()
    }

    @Test
    fun construction_performs_no_request() =
        runTest(testDispatcher) {
            // The container builds this VM before sign-in; an init-time fetch would
            // cache a "Session expired" error that the feed tab then shows post-login.
            viewModel = FeedViewModel(repository)
            advanceUntilIdle()

            assertEquals(0, repository.listItemsCallCount)
        }

    @Test
    fun load_items_success() =
        runTest(testDispatcher) {
            val items = listOf(fakeFeedItem("a"), fakeFeedItem("b"), fakeFeedItem("c"))
            repository.listItemsResult = Result.success(paginatedFeedItems(items))

            viewModel = FeedViewModel(repository)
            viewModel.refresh()
            advanceUntilIdle()

            val state = assertIs<FeedUiState.Success>(viewModel.uiState.value)
            assertEquals(3, state.items.size)
        }

    @Test
    fun load_items_error() =
        runTest(testDispatcher) {
            repository.listItemsResult = Result.failure(RuntimeException("Network error"))

            viewModel = FeedViewModel(repository)
            viewModel.refresh()
            advanceUntilIdle()

            val state = assertIs<FeedUiState.Error>(viewModel.uiState.value)
            assertEquals("Network error", state.message)
        }

    @Test
    fun load_next_page_appends_items_and_passes_cursor() =
        runTest(testDispatcher) {
            repository.listItemsResult =
                Result.success(
                    paginatedFeedItems(
                        listOf(fakeFeedItem("a"), fakeFeedItem("b")),
                        hasMore = true,
                        nextCursor = "cursor-1",
                    ),
                )
            viewModel = FeedViewModel(repository)
            viewModel.refresh()
            advanceUntilIdle()

            repository.listItemsResult =
                Result.success(paginatedFeedItems(listOf(fakeFeedItem("c"), fakeFeedItem("d"))))
            viewModel.loadNextPage()
            advanceUntilIdle()

            val state = assertIs<FeedUiState.Success>(viewModel.uiState.value)
            assertEquals(listOf("a", "b", "c", "d"), state.items.map { it.id })
            assertFalse(state.hasMore)
            assertFalse(state.isLoadingMore)
            assertEquals("cursor-1", repository.lastListItemsCursor)
        }

    @Test
    fun load_next_page_failure_keeps_items_and_emits_snackbar() =
        runTest(testDispatcher) {
            repository.listItemsResult =
                Result.success(
                    paginatedFeedItems(
                        listOf(fakeFeedItem("a"), fakeFeedItem("b")),
                        hasMore = true,
                        nextCursor = "c1",
                    ),
                )
            viewModel = FeedViewModel(repository)
            viewModel.refresh()
            advanceUntilIdle()

            val effects = mutableListOf<FeedEffect>()
            val job = launch { viewModel.effects.toList(effects) }

            repository.listItemsResult = Result.failure(RuntimeException("boom"))
            viewModel.loadNextPage()
            advanceUntilIdle()

            val state = assertIs<FeedUiState.Success>(viewModel.uiState.value)
            assertEquals(listOf("a", "b"), state.items.map { it.id })
            assertFalse(state.isLoadingMore)
            assertTrue(effects.any { it is FeedEffect.ShowSnackbar })
            job.cancel()
        }

    @Test
    fun open_delivery_prepares_and_emits_open_reader() =
        runTest(testDispatcher) {
            repository.listItemsResult = Result.success(FakeFeedRepository.emptyPaginatedFeedItems())
            repository.prepareDeliveryResult = Result.success("doc_42")
            viewModel = FeedViewModel(repository)
            viewModel.refresh()
            advanceUntilIdle()

            val effects = mutableListOf<FeedEffect>()
            val job = launch { viewModel.effects.toList(effects) }

            viewModel.openDelivery("fd_7")
            advanceUntilIdle()

            assertEquals("fd_7", repository.lastPreparedDeliveryId)
            val open = effects.filterIsInstance<FeedEffect.OpenReader>().firstOrNull()
            assertEquals("doc_42", open?.documentId)
            job.cancel()
        }

    @Test
    fun open_delivery_removes_item_from_unseen_list() =
        runTest(testDispatcher) {
            val items = listOf(fakeFeedItem("a"), fakeFeedItem("b"))
            repository.listItemsResult = Result.success(paginatedFeedItems(items))
            repository.prepareDeliveryResult = Result.success("doc_b")
            viewModel = FeedViewModel(repository)
            viewModel.refresh()
            advanceUntilIdle()

            viewModel.openDelivery("b")
            advanceUntilIdle()

            val state = assertIs<FeedUiState.Success>(viewModel.uiState.value)
            assertEquals(listOf("a"), state.items.map { it.id })
        }

    @Test
    fun open_delivery_failure_emits_snackbar_and_no_navigation() =
        runTest(testDispatcher) {
            repository.listItemsResult = Result.success(FakeFeedRepository.emptyPaginatedFeedItems())
            repository.prepareDeliveryResult = Result.failure(RuntimeException("No link to read"))
            viewModel = FeedViewModel(repository)
            viewModel.refresh()
            advanceUntilIdle()

            val effects = mutableListOf<FeedEffect>()
            val job = launch { viewModel.effects.toList(effects) }

            viewModel.openDelivery("fd_7")
            advanceUntilIdle()

            assertTrue(effects.any { it is FeedEffect.ShowSnackbar })
            assertFalse(effects.any { it is FeedEffect.OpenReader })
            job.cancel()
        }

    @Test
    fun initial_filter_is_unseen() =
        runTest(testDispatcher) {
            repository.listItemsResult = Result.success(FakeFeedRepository.emptyPaginatedFeedItems())

            viewModel = FeedViewModel(repository)
            viewModel.refresh()
            advanceUntilIdle()

            assertEquals(FeedFilter.UNSEEN, viewModel.feedFilter.value)
            assertEquals("unseen", repository.lastListItemsState)
        }

    @Test
    fun switch_to_seen_filter() =
        runTest(testDispatcher) {
            repository.listItemsResult = Result.success(FakeFeedRepository.emptyPaginatedFeedItems())

            viewModel = FeedViewModel(repository)
            viewModel.refresh()
            advanceUntilIdle()

            viewModel.setFeedFilter(FeedFilter.SEEN)
            advanceUntilIdle()

            assertEquals(FeedFilter.SEEN, viewModel.feedFilter.value)
            assertEquals("seen", repository.lastListItemsState)
        }

    @Test
    fun mark_seen_removes_item_optimistically() =
        runTest(testDispatcher) {
            val item = fakeFeedItem("target")
            repository.listItemsResult =
                Result.success(paginatedFeedItems(listOf(item, fakeFeedItem("other"))))

            viewModel = FeedViewModel(repository)
            viewModel.refresh()
            advanceUntilIdle()

            viewModel.markSeen(item)

            val state = assertIs<FeedUiState.Success>(viewModel.uiState.value)
            assertTrue(state.items.none { it.id == "target" })
            assertEquals(1, state.items.size)
        }

    @Test
    fun mark_seen_error_restores_item() =
        runTest(testDispatcher) {
            val item = fakeFeedItem("target")
            repository.listItemsResult =
                Result.success(paginatedFeedItems(listOf(item, fakeFeedItem("other"))))
            repository.markSeenResult = Result.failure(RuntimeException("Server error"))

            viewModel = FeedViewModel(repository)
            viewModel.refresh()
            advanceUntilIdle()

            viewModel.markSeen(item)
            advanceUntilIdle()

            val state = assertIs<FeedUiState.Success>(viewModel.uiState.value)
            assertTrue(state.items.any { it.id == "target" })
            assertEquals(2, state.items.size)
        }

    @Test
    fun save_to_library_marks_delivery_id_in_saved_set_optimistically() =
        runTest(testDispatcher) {
            val item = fakeFeedItem("target")
            repository.listItemsResult = Result.success(paginatedFeedItems(listOf(item)))

            viewModel = FeedViewModel(repository)
            viewModel.refresh()
            advanceUntilIdle()

            viewModel.saveToLibrary(item)

            val state = assertIs<FeedUiState.Success>(viewModel.uiState.value)
            assertTrue(state.savedItemIds.contains("target"))
        }

    @Test
    fun save_to_library_error_removes_delivery_id_from_saved_set() =
        runTest(testDispatcher) {
            val item = fakeFeedItem("target")
            repository.listItemsResult = Result.success(paginatedFeedItems(listOf(item)))
            repository.saveToLibraryResult = Result.failure(RuntimeException("Failed"))

            viewModel = FeedViewModel(repository)
            viewModel.refresh()
            advanceUntilIdle()

            viewModel.saveToLibrary(item)
            advanceUntilIdle()

            val state = assertIs<FeedUiState.Success>(viewModel.uiState.value)
            assertFalse(state.savedItemIds.contains("target"))
        }

    @Test
    fun mark_all_seen_clears_list_optimistically() =
        runTest(testDispatcher) {
            val items = listOf(fakeFeedItem("a"), fakeFeedItem("b"))
            repository.listItemsResult = Result.success(paginatedFeedItems(items))

            viewModel = FeedViewModel(repository)
            viewModel.refresh()
            advanceUntilIdle()

            viewModel.markAllSeen()

            val state = assertIs<FeedUiState.Success>(viewModel.uiState.value)
            assertTrue(state.items.isEmpty())
            assertEquals(1, repository.markAllSeenCallCount)
        }

    @Test
    fun mark_all_seen_error_restores_items() =
        runTest(testDispatcher) {
            val items = listOf(fakeFeedItem("a"), fakeFeedItem("b"))
            repository.listItemsResult = Result.success(paginatedFeedItems(items))
            repository.markAllSeenResult = Result.failure(RuntimeException("Failed"))

            viewModel = FeedViewModel(repository)
            viewModel.refresh()
            advanceUntilIdle()

            viewModel.markAllSeen()
            advanceUntilIdle()

            val state = assertIs<FeedUiState.Success>(viewModel.uiState.value)
            assertEquals(2, state.items.size)
        }

    @Test
    fun load_next_page_appends() =
        runTest(testDispatcher) {
            val page1 = listOf(fakeFeedItem("a"), fakeFeedItem("b"))
            val page2 = listOf(fakeFeedItem("c"), fakeFeedItem("d"))

            repository.listItemsResult =
                Result.success(paginatedFeedItems(page1, hasMore = true, nextCursor = "cursor1"))

            viewModel = FeedViewModel(repository)
            viewModel.refresh()
            advanceUntilIdle()

            repository.listItemsResult = Result.success(paginatedFeedItems(page2, hasMore = false))
            viewModel.loadNextPage()
            advanceUntilIdle()

            val state = assertIs<FeedUiState.Success>(viewModel.uiState.value)
            assertEquals(4, state.items.size)
            assertEquals("cursor1", repository.lastListItemsCursor)
        }

    @Test
    fun empty_state_checks_for_subscriptions() =
        runTest(testDispatcher) {
            repository.listItemsResult = Result.success(FakeFeedRepository.emptyPaginatedFeedItems())
            repository.listSubscriptionsResult =
                Result.success(
                    FakeFeedRepository.emptyPaginatedSubscriptions(),
                )

            viewModel = FeedViewModel(repository)
            viewModel.refresh()
            advanceUntilIdle()

            val state = assertIs<FeedUiState.Success>(viewModel.uiState.value)
            assertTrue(state.items.isEmpty())
            assertTrue(!state.hasSubscriptions)
        }

    @Test
    fun empty_state_with_subscriptions_shows_caught_up() =
        runTest(testDispatcher) {
            repository.listItemsResult = Result.success(FakeFeedRepository.emptyPaginatedFeedItems())
            repository.listSubscriptionsResult =
                Result.success(
                    app.indelible.feed.model.PaginatedSubscriptions(
                        data = listOf(fakeSubscription()),
                        page =
                            app.indelible.core.model
                                .PageInfo(nextCursor = null, hasMore = false),
                    ),
                )

            viewModel = FeedViewModel(repository)
            viewModel.refresh()
            advanceUntilIdle()

            val state = assertIs<FeedUiState.Success>(viewModel.uiState.value)
            assertTrue(state.items.isEmpty())
            assertTrue(state.hasSubscriptions)
        }

    @Test
    fun refresh_reloads_items() =
        runTest(testDispatcher) {
            val items = listOf(fakeFeedItem("a"))
            repository.listItemsResult = Result.success(paginatedFeedItems(items))

            viewModel = FeedViewModel(repository)
            viewModel.refresh()
            advanceUntilIdle()

            val initialCallCount = repository.listItemsCallCount

            viewModel.refresh()
            advanceUntilIdle()

            assertTrue(repository.listItemsCallCount > initialCallCount)
        }
}
