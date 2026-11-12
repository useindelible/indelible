package app.indelible.feed.ui

import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.runComposeUiTest
import app.indelible.feed.viewmodel.FakeFeedRepository
import app.indelible.feed.viewmodel.FeedViewModel
import app.indelible.ui.theme.AppTheme
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlin.test.Test
import kotlin.test.assertEquals

@OptIn(ExperimentalTestApi::class)
class FeedScreenReloadTest {
    private val repository = FakeFeedRepository()
    private val online = MutableStateFlow(true)
    private val reachable = MutableStateFlow<Boolean?>(false)

    private fun failedFeed(): FeedViewModel {
        repository.listItemsResult = Result.failure(IllegalStateException("no answer"))
        return FeedViewModel(repository).also { it.refresh() }
    }

    @Test
    fun theFeedLoadsAgainWhenTheServerAnswers() =
        runComposeUiTest {
            val viewModel = failedFeed()
            setContent { AppTheme { feed(viewModel, online, reachable) } }
            onNodeWithText("Couldn’t load the feed. Please try again.").assertIsDisplayed()
            val before = repository.listItemsCallCount

            runOnIdle { reachable.value = true }

            waitUntil { repository.listItemsCallCount == before + 1 }
        }

    @Test
    fun aFailedFeedOffersRetry() =
        runComposeUiTest {
            val viewModel = failedFeed()
            setContent { AppTheme { feed(viewModel, online, reachable) } }
            val before = repository.listItemsCallCount

            onNodeWithText("Retry").performClick()

            waitUntil { repository.listItemsCallCount == before + 1 }
            assertEquals(before + 1, repository.listItemsCallCount)
        }
}

@androidx.compose.runtime.Composable
private fun feed(
    viewModel: FeedViewModel,
    online: Flow<Boolean>,
    reachable: Flow<Boolean?>,
) {
    FeedScreen(
        viewModel = viewModel,
        onNavigateToAddFeed = {},
        onMenuClick = {},
        onProfileClick = {},
        onManageSources = {},
        onNavigateToReader = {},
        online = online,
        reachable = reachable,
    )
}
