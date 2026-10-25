package app.indelible.reader.ui.components

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.size
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.LocalSaveableStateRegistry
import androidx.compose.runtime.saveable.SaveableStateRegistry
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.assertIsNotEnabled
import androidx.compose.ui.test.hasSetTextAction
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performTextClearance
import androidx.compose.ui.test.performTextInput
import androidx.compose.ui.test.runComposeUiTest
import androidx.compose.ui.unit.dp
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
    fun saveIsDisabledForAnUnchangedNote() =
        runComposeUiTest {
            val stored = fakeHighlight().copy(note = fakeNote().copy(body = "Remember this\n"))
            setContent { AppTheme { HighlightNoteEditor(stored, onSave = {}, onDeleteNote = {}) } }

            onNodeWithText("Save").assertIsNotEnabled()
        }

    @Test
    fun noteAddedElsewhereWhileEditingIsNotDeletedBySave() =
        runComposeUiTest {
            var highlight by mutableStateOf(fakeHighlight())
            var deleted = 0
            setContent { AppTheme { HighlightNoteEditor(highlight, onSave = {}, onDeleteNote = { deleted++ }) } }

            highlight = highlight.copy(note = fakeNote().copy(body = "From the web"))
            waitForIdle()
            onNodeWithText("Save").performClick()

            assertEquals(0, deleted)
        }

    @Test
    fun deleteNoteAsksForTheDeletion() =
        runComposeUiTest {
            var deleted = 0
            setContent { AppTheme { HighlightNoteEditor(noted, onSave = {}, onDeleteNote = { deleted++ }) } }

            onNodeWithText("Delete note").performClick()

            assertEquals(1, deleted)
        }

    @Test
    fun longNoteKeepsSaveVisible() =
        runComposeUiTest {
            setContent {
                AppTheme {
                    Box(Modifier.size(400.dp, 400.dp)) {
                        HighlightNoteEditor(fakeHighlight(), onSave = {}, onDeleteNote = {})
                    }
                }
            }

            onNode(hasSetTextAction()).performTextInput("line\n".repeat(80))

            onNodeWithText("Save").assertIsDisplayed()
        }

    @Test
    fun typedNoteSurvivesRecreation() =
        runComposeUiTest {
            var state: Map<String, List<Any?>>? = null
            var hidden by mutableStateOf(false)
            var generation by mutableStateOf(0)
            val registries = mutableListOf<SaveableStateRegistry>()
            setContent {
                val registry = remember(generation) { SaveableStateRegistry(state) { true } }
                registries += registry
                CompositionLocalProvider(LocalSaveableStateRegistry provides registry) {
                    if (!hidden) {
                        AppTheme { HighlightNoteEditor(fakeHighlight(), onSave = {}, onDeleteNote = {}) }
                    }
                }
            }

            onNode(hasSetTextAction()).performTextInput("A thought")
            state = registries.last().performSave()
            hidden = true
            waitForIdle()
            generation++
            hidden = false
            waitForIdle()

            onNodeWithText("A thought").assertIsDisplayed()
        }
}
