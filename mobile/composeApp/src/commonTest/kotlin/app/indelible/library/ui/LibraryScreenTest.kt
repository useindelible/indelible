package app.indelible.library.ui

import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.hasAnyDescendant
import androidx.compose.ui.test.hasScrollAction
import androidx.compose.ui.test.hasText
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performScrollToIndex
import androidx.compose.ui.test.runComposeUiTest
import app.indelible.core.model.LibraryCounts
import app.indelible.library.viewmodel.FakeLibraryRepository
import app.indelible.library.viewmodel.FakeLibraryRepository.Companion.fakeLibraryItem
import app.indelible.library.viewmodel.FakeLibraryRepository.Companion.paginatedItems
import app.indelible.library.viewmodel.LibraryScope
import app.indelible.library.viewmodel.LibraryViewModel
import app.indelible.profile.repository.AddLibraryRepository
import app.indelible.profile.viewmodel.AddLibraryViewModel
import app.indelible.ui.theme.AppTheme
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
            val addLibraryViewModel =
                AddLibraryViewModel(
                    object : AddLibraryRepository {
                        override suspend fun save(url: String): Result<Unit> = Result.success(Unit)
                    },
                )
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

    private fun counts(total: Int) =
        LibraryCounts(
            total = total,
            unread = total,
            reading = 0,
            done = 0,
            byItemType = emptyMap(),
        )
}
