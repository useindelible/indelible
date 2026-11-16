package app.indelible.reader.ui.components

import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.runComposeUiTest
import app.indelible.reader.model.ReaderDocument
import app.indelible.ui.theme.AppTheme
import kotlin.test.Test

@OptIn(ExperimentalTestApi::class)
class ItemRecordPanelTest {
    @Test
    fun theRecordOpensOnItsInfoWithNoActionsOrOfflineSection() =
        runComposeUiTest {
            val item =
                ReaderDocument(
                    libraryEntryId = "lib_1",
                    id = "doc_1",
                    itemType = "article",
                    title = "Title",
                    url = "https://example.test/a",
                    saved = true,
                    readableReady = true,
                    availableAssets = listOf("readable_html"),
                )
            setContent {
                AppTheme {
                    ItemRecordPanel(
                        item = item,
                        note = null,
                        tags = emptyList(),
                        availableTags = emptyList(),
                        highlights = emptyList(),
                        progress = 0f,
                        onEditNote = {},
                        onTagsChanged = {},
                    )
                }
            }

            onNodeWithText("Info").assertIsDisplayed()
            onNodeWithText("Actions").assertDoesNotExist()
            onNodeWithText("Share link").assertDoesNotExist()
            onNodeWithText("Offline").assertDoesNotExist()
            onNodeWithText("Keep offline").assertDoesNotExist()
        }
}
