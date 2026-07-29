package app.indelible.library.ui.components

import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.width
import androidx.compose.ui.Modifier
import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.runComposeUiTest
import androidx.compose.ui.unit.dp
import app.indelible.library.viewmodel.TriageFilter
import app.indelible.ui.theme.AppTheme
import kotlin.test.Test

@OptIn(ExperimentalTestApi::class)
class LibraryEmptyStateTest {
    @Test
    fun empty_inbox_preserves_the_first_item_slot_and_explains_the_layout() =
        runComposeUiTest {
            setContent {
                AppTheme {
                    LibraryEmptyState(
                        triageFilter = TriageFilter.INBOX,
                        modifier = Modifier.width(360.dp).height(480.dp),
                    )
                }
            }

            onNodeWithText("First save").assertIsDisplayed()
            onNodeWithText("Save a link and it appears right here").assertIsDisplayed()
            onNodeWithText("Saved items land in this list").assertIsDisplayed()
        }
}
