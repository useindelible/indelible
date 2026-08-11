package app.indelible.navigation

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.runComposeUiTest
import kotlin.test.Test
import kotlin.test.assertEquals

@OptIn(ExperimentalTestApi::class)
class MainNavigationTest {
    @Test
    fun returning_to_library_refreshes_each_time() =
        runComposeUiTest {
            var route by mutableStateOf(TabItem.HOME.route)
            var refreshes = 0

            setContent {
                RefreshLibraryOnDestination(route) { refreshes += 1 }
            }

            runOnIdle { route = TabItem.LIBRARY.route }
            waitForIdle()
            assertEquals(1, refreshes)

            runOnIdle { route = TabItem.FEED.route }
            waitForIdle()
            runOnIdle { route = TabItem.LIBRARY.route }
            waitForIdle()
            assertEquals(2, refreshes)
        }
}
