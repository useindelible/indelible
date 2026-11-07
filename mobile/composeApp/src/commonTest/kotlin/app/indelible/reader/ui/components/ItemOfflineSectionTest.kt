package app.indelible.reader.ui.components

import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.assertIsOn
import androidx.compose.ui.test.isToggleable
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.runComposeUiTest
import app.indelible.core.offline.Acquisition
import app.indelible.core.offline.DownloadFailure
import app.indelible.offline.viewmodel.Availability
import app.indelible.offline.viewmodel.DocumentOfflineStatus
import app.indelible.reader.model.ReaderDocument
import app.indelible.ui.theme.AppTheme
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

@OptIn(ExperimentalTestApi::class)
class ItemOfflineSectionTest {
    @Test
    fun noSpaceLinksToStorage() =
        runComposeUiTest {
            var opened = false
            val status = DocumentOfflineStatus(acquisition = Acquisition.Failed(DownloadFailure.NO_SPACE))
            setContent {
                AppTheme { ItemOfflineSection(ItemOffline(status, onOpenStorage = { opened = true })) }
            }

            onNodeWithText("Not enough space").assertIsDisplayed()
            onNodeWithText("Manage storage").performClick()

            assertTrue(opened)
        }

    @Test
    fun recordPanelShowsTheOfflineSection() =
        runComposeUiTest {
            val item =
                ReaderDocument(
                    libraryEntryId = "lib_1",
                    id = "doc_1",
                    itemType = "article",
                    title = "Title",
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
                        offline = ItemOffline(DocumentOfflineStatus()),
                    )
                }
            }

            onNodeWithText("Keep offline").assertIsDisplayed()
        }

    @Test
    fun keptCopyShowsItsSizeAndCanBeTurnedOffOrRemoved() =
        runComposeUiTest {
            var kept: Boolean? = null
            var removed = false
            val status = DocumentOfflineStatus(Availability.KEPT, bytes = 2_048L)
            setContent {
                AppTheme {
                    ItemOfflineSection(
                        ItemOffline(status, onKeepOffline = { kept = it }, onRemove = { removed = true }),
                    )
                }
            }

            onNodeWithText("2 KB · not synced yet").assertIsDisplayed()
            onNode(isToggleable()).assertIsOn().performClick()
            onNodeWithText("Remove from device").performClick()

            assertEquals(false, kept)
            assertTrue(removed)
        }
}
