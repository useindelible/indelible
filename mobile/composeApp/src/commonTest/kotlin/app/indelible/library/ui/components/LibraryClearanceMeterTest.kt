package app.indelible.library.ui.components

import androidx.compose.foundation.layout.width
import androidx.compose.ui.Modifier
import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.runComposeUiTest
import androidx.compose.ui.unit.dp
import app.indelible.core.model.LibraryCounts
import app.indelible.ui.theme.AppTheme
import kotlin.test.Test

@OptIn(ExperimentalTestApi::class)
class LibraryClearanceMeterTest {
    @Test
    fun empty_scope_keeps_the_zero_rail_and_legend_visible() =
        runComposeUiTest {
            setContent {
                AppTheme {
                    LibraryClearanceMeter(
                        counts =
                            LibraryCounts(
                                total = 0,
                                unread = 0,
                                reading = 0,
                                done = 0,
                                byItemType = emptyMap(),
                            ),
                        modifier = Modifier.width(360.dp),
                    )
                }
            }

            onNodeWithContentDescription("0 unread, 0 reading, nothing to clear").assertIsDisplayed()
            onNodeWithText("0 unread").assertIsDisplayed()
            onNodeWithText("Nothing to clear").assertIsDisplayed()
        }
}
