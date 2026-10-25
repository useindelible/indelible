package app.indelible.reader.ui.components

import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.hasSetTextAction
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performTextClearance
import androidx.compose.ui.test.performTextInput
import androidx.compose.ui.test.runComposeUiTest
import app.indelible.reader.viewmodel.FakeReaderRepository.Companion.fakeHighlight
import app.indelible.reader.viewmodel.FakeReaderRepository.Companion.fakeNote
import app.indelible.ui.theme.AppTheme
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull

@OptIn(ExperimentalTestApi::class)
class HighlightNoteSheetTest {
    private val noted = fakeHighlight().copy(note = fakeNote().copy(body = "Remember this"))

    @Test
    fun existingNoteIsPrefilled() =
        runComposeUiTest {
            setContent { AppTheme { HighlightNoteEditor(noted, onSave = {}, onDeleteNote = {}) } }

            onNodeWithText("Remember this").assertIsDisplayed()
            onNodeWithText("Delete note").assertIsDisplayed()
        }

    @Test
    fun savingSendsTheBody() =
        runComposeUiTest {
            var saved: String? = null
            setContent { AppTheme { HighlightNoteEditor(fakeHighlight(), onSave = { saved = it }, onDeleteNote = {}) } }

            onNode(hasSetTextAction()).performTextInput("A thought ")
            onNodeWithText("Save").performClick()

            assertEquals("A thought", saved)
        }

    @Test
    fun clearingTheNoteDeletesIt() =
        runComposeUiTest {
            var saved: String? = null
            var deleted = 0
            setContent {
                AppTheme { HighlightNoteEditor(noted, onSave = { saved = it }, onDeleteNote = { deleted++ }) }
            }

            onNode(hasSetTextAction()).performTextClearance()
            onNodeWithText("Save").performClick()

            assertEquals(1, deleted)
            assertNull(saved)
        }

    @Test
    fun deleteNoteAsksForTheDeletion() =
        runComposeUiTest {
            var deleted = 0
            setContent { AppTheme { HighlightNoteEditor(noted, onSave = {}, onDeleteNote = { deleted++ }) } }

            onNodeWithText("Delete note").performClick()

            assertEquals(1, deleted)
        }
}
