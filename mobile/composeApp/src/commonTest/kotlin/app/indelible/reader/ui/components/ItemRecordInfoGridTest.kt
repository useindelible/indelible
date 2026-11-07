package app.indelible.reader.ui.components

import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.runComposeUiTest
import app.indelible.reader.model.ReaderDocument
import app.indelible.ui.theme.AppTheme
import kotlinx.datetime.Instant
import kotlin.test.Test

@OptIn(ExperimentalTestApi::class)
class ItemRecordInfoGridTest {
    private fun document(savedAt: Instant?) =
        ReaderDocument(
            id = "doc_1",
            itemType = "article",
            title = "Title",
            saved = true,
            readableReady = true,
            availableAssets = emptyList(),
            lastReadAt = Instant.parse("2026-04-01T12:00:00Z"),
            savedAt = savedAt,
        )

    @Test
    fun aKnownSavedDateIsShown() =
        runComposeUiTest {
            setContent { AppTheme { InfoGrid(document(Instant.parse("2026-03-01T12:00:00Z")), progress = 0f) } }

            onNodeWithText("Saved").assertIsDisplayed()
        }

    @Test
    fun anUnknownSavedDateHidesTheRow() =
        runComposeUiTest {
            setContent { AppTheme { InfoGrid(document(savedAt = null), progress = 0f) } }

            onNodeWithText("Saved").assertDoesNotExist()
        }
}
