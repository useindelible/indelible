package app.indelible.feed.ui

import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.width
import androidx.compose.ui.Modifier
import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.assertCountEquals
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.onAllNodesWithText
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.runComposeUiTest
import androidx.compose.ui.unit.dp
import app.indelible.feed.viewmodel.FeedFilter
import app.indelible.ui.theme.AppTheme
import kotlin.test.Test
import kotlin.test.assertTrue

@OptIn(ExperimentalTestApi::class)
class FeedEmptyStateTest {
    @Test
    fun feed_without_sources_preserves_the_list_and_opens_add_feed() =
        runComposeUiTest {
            var addRequested = false
            setContent {
                AppTheme {
                    FeedEmptyState(
                        filter = FeedFilter.UNSEEN,
                        hasSubscriptions = false,
                        onAddFeed = { addRequested = true },
                        modifier = Modifier.width(360.dp).height(480.dp),
                    )
                }
            }

            onNodeWithText("First source").assertIsDisplayed()
            onNodeWithText("Follow a source and new articles appear here").assertIsDisplayed()
            onNodeWithText("New posts land in this list").assertIsDisplayed()
            onNodeWithText("Add a feed").performClick()

            runOnIdle { assertTrue(addRequested) }
        }

    @Test
    fun caught_up_feed_explains_the_zero_without_showing_an_add_action() =
        runComposeUiTest {
            setContent {
                AppTheme {
                    FeedEmptyState(
                        filter = FeedFilter.UNSEEN,
                        hasSubscriptions = true,
                        onAddFeed = {},
                        modifier = Modifier.width(360.dp).height(480.dp),
                    )
                }
            }

            onNodeWithText("No unseen posts").assertIsDisplayed()
            onNodeWithText("New posts from your sources will appear here.").assertIsDisplayed()
            onAllNodesWithText("Add a feed").assertCountEquals(0)
        }
}
