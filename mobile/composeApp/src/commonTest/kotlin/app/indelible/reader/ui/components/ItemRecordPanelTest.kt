package app.indelible.reader.ui.components

import androidx.compose.runtime.Composable
import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.runComposeUiTest
import app.indelible.offline.viewmodel.Availability
import app.indelible.offline.viewmodel.DocumentOfflineStatus
import app.indelible.reader.model.ReaderDocument
import app.indelible.ui.theme.AppTheme
import kotlin.test.Test
import kotlin.test.assertTrue

private val ITEM =
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

@OptIn(ExperimentalTestApi::class)
class ItemRecordPanelTest {
    @Composable
    private fun Record(offline: ItemOffline? = null) {
        AppTheme {
            ItemRecordPanel(
                item = ITEM,
                note = null,
                tags = emptyList(),
                availableTags = emptyList(),
                highlights = emptyList(),
                progress = 0f,
                onEditNote = {},
                onTagsChanged = {},
                offline = offline,
            )
        }
    }

    @Test
    fun theRecordOpensOnItsInfoWithNoActionsOrOfflineSection() =
        runComposeUiTest {
            setContent { Record() }

            onNodeWithText("Info").assertIsDisplayed()
            onNodeWithText("Actions").assertDoesNotExist()
            onNodeWithText("Share link").assertDoesNotExist()
            onNodeWithText("Offline").assertDoesNotExist()
            onNodeWithText("Keep on this phone").assertDoesNotExist()
        }

    @Test
    fun aDocumentNotOnThePhoneOffersKeep() =
        runComposeUiTest {
            var kept = false
            setContent { Record(ItemOffline(DocumentOfflineStatus(), onKeep = { kept = true })) }

            onNodeWithText("Keep on this phone").performClick()

            assertTrue(kept)
        }

    @Test
    fun aKeptDocumentOffersNoKeep() =
        runComposeUiTest {
            setContent { Record(ItemOffline(DocumentOfflineStatus(availability = Availability.KEPT))) }

            onNodeWithText("Keep on this phone").assertDoesNotExist()
        }
}
