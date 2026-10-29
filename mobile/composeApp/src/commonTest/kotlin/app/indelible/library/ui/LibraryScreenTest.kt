package app.indelible.library.ui

import androidx.compose.ui.test.ComposeUiTest
import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.assertCountEquals
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.hasAnyDescendant
import androidx.compose.ui.test.hasScrollAction
import androidx.compose.ui.test.hasText
import androidx.compose.ui.test.onAllNodesWithText
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performScrollToIndex
import androidx.compose.ui.test.runComposeUiTest
import app.indelible.core.model.LibraryCounts
import app.indelible.core.model.PaginatedItems
import app.indelible.library.viewmodel.FakeLibraryRepository
import app.indelible.library.viewmodel.FakeLibraryRepository.Companion.fakeLibraryItem
import app.indelible.library.viewmodel.FakeLibraryRepository.Companion.paginatedItems
import app.indelible.library.viewmodel.LibraryScope
import app.indelible.library.viewmodel.LibraryViewModel
import app.indelible.offline.viewmodel.Availability
import app.indelible.offline.viewmodel.DocumentOfflineStatus
import app.indelible.offline.viewmodel.SyncBadge
import app.indelible.profile.repository.AddLibraryRepository
import app.indelible.profile.viewmodel.AddLibraryViewModel
import app.indelible.ui.theme.AppTheme
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.emptyFlow
import kotlinx.coroutines.flow.flowOf
import kotlin.test.Test
import kotlin.test.assertEquals

@OptIn(ExperimentalTestApi::class)
class LibraryScreenTest {
    @Test
    fun authoritative_refresh_reveals_a_new_first_item() =
        runComposeUiTest {
            val repository = FakeLibraryRepository()
            val originalItems = (0..11).map { fakeLibraryItem("item$it") }
            repository.listItemsResult = Result.success(paginatedItems(originalItems))
            val viewModel = LibraryViewModel(repository)
            val addLibraryViewModel = addLibraryViewModel()
            viewModel.refresh()

            setContent {
                AppTheme {
                    LibraryScreen(
                        viewModel = viewModel,
                        addLibraryViewModel = addLibraryViewModel,
                        onNavigateToItem = {},
                        onMenuClick = {},
                        onProfileClick = {},
                        collections = emptyList(),
                        smartLists = emptyList(),
                    )
                }
            }

            onNode(hasScrollAction() and hasAnyDescendant(hasText("Test Article item0"))).performScrollToIndex(11)
            onNodeWithText("Test Article item11").assertIsDisplayed()
            runOnIdle {
                repository.listItemsResult =
                    Result.success(paginatedItems(listOf(fakeLibraryItem("new")) + originalItems))
                viewModel.refresh()
            }

            waitUntil { repository.listItemsCallCount == 2 }
            onNodeWithText("Test Article new").assertIsDisplayed()
        }

    @Test
    fun offline_badges_show_on_rows_the_device_knows() =
        runComposeUiTest {
            val repository = FakeLibraryRepository()
            repository.listItemsResult =
                Result.success(paginatedItems(listOf(fakeLibraryItem("kept"), fakeLibraryItem("other"))))
            val statuses = mapOf("doc_kept" to DocumentOfflineStatus(Availability.KEPT, SyncBadge.Pending(2)))
            val viewModel = LibraryViewModel(repository, flowOf(statuses))
            viewModel.refresh()

            setContent {
                AppTheme {
                    LibraryScreen(
                        viewModel = viewModel,
                        addLibraryViewModel = addLibraryViewModel(),
                        onNavigateToItem = {},
                        onMenuClick = {},
                        onProfileClick = {},
                        collections = emptyList(),
                        smartLists = emptyList(),
                    )
                }
            }

            onNodeWithText("Test Article other").assertIsDisplayed()
            onAllNodesWithText("Kept").assertCountEquals(1)
            onNodeWithText("2 pending").assertIsDisplayed()
        }

    @Test
    fun triage_scope_keeps_its_zero_count_visible() {
        val emptyCounts = counts(total = 0)

        assertEquals(
            0,
            scopeCount(
                scope = LibraryScope.Triage,
                collections = emptyList(),
                counts = emptyCounts,
            ),
        )
    }

    @Test
    fun deviceListShowsTheLastLoadedNotice() =
        runComposeUiTest {
            val repository = FakeLibraryRepository()
            repository.listItemsResult = Result.success(devicePage("kept"))
            show(loaded(repository))

            onNodeWithText("Showing what was last loaded.").assertIsDisplayed()
            onNodeWithText("Test Article kept").assertIsDisplayed()
        }

    @Test
    fun serverListShowsNoNotice() =
        runComposeUiTest {
            val repository = FakeLibraryRepository()
            repository.listItemsResult = Result.success(paginatedItems(listOf(fakeLibraryItem("a"))))
            show(loaded(repository))

            onNodeWithText("Test Article a").assertIsDisplayed()
            onAllNodesWithText("Showing what was last loaded.").assertCountEquals(0)
        }

    @Test
    fun errorOffersRetry() =
        runComposeUiTest {
            val repository = FakeLibraryRepository()
            repository.listItemsResult = Result.failure(IllegalStateException("offline"))
            show(loaded(repository))

            repository.listItemsResult = Result.success(paginatedItems(listOf(fakeLibraryItem("back"))))
            onNodeWithText("Retry").performClick()

            waitUntil { repository.listItemsCallCount == 2 }
            onNodeWithText("Test Article back").assertIsDisplayed()
        }

    @Test
    fun networkBackRefreshesADeviceList() =
        runComposeUiTest {
            val repository = FakeLibraryRepository()
            repository.listItemsResult = Result.success(devicePage("kept"))
            val online = MutableStateFlow(false)
            show(loaded(repository), online = online)
            onNodeWithText("Test Article kept").assertIsDisplayed()

            repository.listItemsResult = Result.success(paginatedItems(listOf(fakeLibraryItem("fresh"))))
            online.value = true

            waitUntil { repository.listItemsCallCount == 2 }
            onNodeWithText("Test Article fresh").assertIsDisplayed()
            onAllNodesWithText("Showing what was last loaded.").assertCountEquals(0)
        }

    @Test
    fun bothSignalsDuringOneDelayedRequestSendOneRequest() =
        runComposeUiTest {
            val repository = FakeLibraryRepository()
            val fresh = CompletableDeferred<Result<PaginatedItems>>()
            repository.listItemsAnswer = { call ->
                if (call == 1) Result.success(devicePage("kept")) else fresh.await()
            }
            val online = MutableStateFlow(false)
            val reachable = MutableStateFlow<Boolean?>(false)
            show(loaded(repository), online, reachable)
            onNodeWithText("Test Article kept").assertIsDisplayed()

            online.value = true
            waitUntil { repository.listItemsCallCount == 2 }
            reachable.value = true
            waitForIdle()
            fresh.complete(Result.success(paginatedItems(listOf(fakeLibraryItem("fresh")))))

            waitUntil { onAllNodesWithText("Test Article fresh").fetchSemanticsNodes().isNotEmpty() }
            assertEquals(2, repository.listItemsCallCount)
        }

    @Test
    fun countsGettingThroughAfterAFailedListReloadOnlyOnce() =
        runComposeUiTest {
            val repository = FakeLibraryRepository()
            val reachable = MutableStateFlow<Boolean?>(false)
            repository.listItemsAnswer = {
                reachable.value = false
                Result.success(devicePage("kept"))
            }
            repository.scopeCountsAnswer = { call ->
                if (call <= COUNTS_THAT_GET_THROUGH) reachable.value = true
                Result.success(counts(1))
            }
            val viewModel = LibraryViewModel(repository)
            show(viewModel, reachable = reachable)

            runOnUiThread { viewModel.refresh() }
            waitUntil { repository.scopeCountsCallCount >= 1 && repository.listItemsCallCount >= 2 }
            waitForIdle()

            assertEquals(2, repository.listItemsCallCount)
            assertEquals(1, repository.scopeCountsCallCount)
        }

    @Test
    fun networkBackLeavesAServerListAlone() =
        runComposeUiTest {
            val repository = FakeLibraryRepository()
            repository.listItemsResult = Result.success(paginatedItems(listOf(fakeLibraryItem("a"))))
            val online = MutableStateFlow(false)
            show(loaded(repository), online = online)
            onNodeWithText("Test Article a").assertIsDisplayed()

            online.value = true
            waitForIdle()

            assertEquals(1, repository.listItemsCallCount)
        }

    private fun loaded(repository: FakeLibraryRepository) = LibraryViewModel(repository).apply { refresh() }

    private fun devicePage(id: String) = paginatedItems(listOf(fakeLibraryItem(id))).copy(fromDevice = true)

    private fun ComposeUiTest.show(
        viewModel: LibraryViewModel,
        online: Flow<Boolean> = emptyFlow(),
        reachable: Flow<Boolean?> = emptyFlow(),
    ) {
        setContent {
            AppTheme {
                LibraryScreen(
                    viewModel = viewModel,
                    addLibraryViewModel = addLibraryViewModel(),
                    onNavigateToItem = {},
                    onMenuClick = {},
                    onProfileClick = {},
                    collections = emptyList(),
                    smartLists = emptyList(),
                    online = online,
                    reachable = reachable,
                )
            }
        }
    }

    private fun counts(total: Int) =
        LibraryCounts(
            total = total,
            unread = total,
            reading = 0,
            done = 0,
            byItemType = emptyMap(),
        )

    private companion object {
        const val COUNTS_THAT_GET_THROUGH = 3
    }

    private fun addLibraryViewModel() =
        AddLibraryViewModel(
            object : AddLibraryRepository {
                override suspend fun save(url: String): Result<Unit> = Result.success(Unit)
            },
        )
}
