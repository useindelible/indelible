package app.indelible.reader.ui.components

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.size
import androidx.compose.runtime.Composable
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
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performTextClearance
import androidx.compose.ui.test.performTextInput
import androidx.compose.ui.test.runComposeUiTest
import androidx.compose.ui.unit.dp
import app.indelible.reader.model.HighlightData
import app.indelible.reader.viewmodel.FakeReaderRepository.Companion.fakeHighlight
import app.indelible.reader.viewmodel.FakeReaderRepository.Companion.fakeNote
import app.indelible.ui.theme.AppTheme
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNotNull
import kotlin.test.assertNull
import kotlin.test.assertTrue

@OptIn(ExperimentalTestApi::class)
class HighlightNoteSheetTest {
    private val noted = fakeHighlight().copy(note = fakeNote().copy(body = "Remember this"))

    @Test
    fun existingNoteIsPrefilled() =
        runComposeUiTest {
            setContent { AppTheme { NoteEditor(noted, onSave = {}, onDeleteNote = {}) } }

            onNodeWithText("Remember this").assertIsDisplayed()
            onNodeWithText("Delete note").assertIsDisplayed()
        }

    @Test
    fun savingSendsTheBody() =
        runComposeUiTest {
            var saved: String? = null
            setContent { AppTheme { NoteEditor(fakeHighlight(), onSave = { saved = it }, onDeleteNote = {}) } }

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
                AppTheme { NoteEditor(noted, onSave = { saved = it }, onDeleteNote = { deleted++ }) }
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
            setContent {
                AppTheme { HighlightNoteSheet(stored, onSave = { _, _ -> }, onDeleteNote = {}, onDismiss = {}) }
            }

            onNodeWithText("Save").assertIsNotEnabled()
        }

    @Test
    fun noteAddedElsewhereWhileEditingIsNotDeletedBySave() =
        runComposeUiTest {
            var highlight by mutableStateOf(fakeHighlight())
            var deleted = 0
            setContent {
                AppTheme {
                    HighlightNoteSheet(highlight, onSave = { _, _ -> }, onDeleteNote = { deleted++ }, onDismiss = {})
                }
            }

            highlight = highlight.copy(note = fakeNote().copy(body = "From the web"))
            waitForIdle()
            onNodeWithText("Save").performClick()

            assertEquals(0, deleted)
        }

    @Test
    fun deleteNoteAsksForTheDeletion() =
        runComposeUiTest {
            var deleted = 0
            setContent { AppTheme { NoteEditor(noted, onSave = {}, onDeleteNote = { deleted++ }) } }

            onNodeWithText("Delete note").performClick()

            assertEquals(1, deleted)
        }

    @Test
    fun deletingTheNoteClosesTheSheet() =
        runComposeUiTest {
            var deleted: (() -> Unit)? = null
            var dismissed = false
            setContent {
                AppTheme {
                    HighlightNoteSheet(noted, { _, _ -> }, { done -> deleted = done }, onDismiss = { dismissed = true })
                }
            }

            onNodeWithText("Delete note").performClick()
            assertNotNull(deleted).invoke()
            waitForIdle()

            assertTrue(dismissed)
        }

    @Test
    fun deleteThatCompletesAfterMoreTypingKeepsTheEditorOpen() =
        runComposeUiTest {
            var deleted: (() -> Unit)? = null
            var dismissed = false
            setContent {
                AppTheme {
                    HighlightNoteSheet(noted, { _, _ -> }, { done -> deleted = done }, onDismiss = { dismissed = true })
                }
            }

            onNodeWithText("Delete note").performClick()
            onNode(hasSetTextAction()).performTextInput("New ")
            assertNotNull(deleted).invoke()
            waitForIdle()

            assertFalse(dismissed)
        }

    @Test
    fun longNoteKeepsSaveVisible() =
        runComposeUiTest {
            setContent {
                AppTheme {
                    Box(Modifier.size(400.dp, 400.dp)) {
                        NoteEditor(fakeHighlight(), onSave = {}, onDeleteNote = {})
                    }
                }
            }

            onNode(hasSetTextAction()).performTextInput("line\n".repeat(80))

            onNodeWithText("Save").assertIsDisplayed()
        }

    @Test
    fun saveStaysVisibleInAShortViewport() =
        runComposeUiTest {
            setContent {
                AppTheme {
                    Box(Modifier.size(400.dp, 260.dp)) {
                        NoteEditor(fakeHighlight(), onSave = {}, onDeleteNote = {})
                    }
                }
            }

            onNode(hasSetTextAction()).performTextInput("line\n".repeat(80))

            onNodeWithText("Save").assertIsDisplayed()
        }

    @Test
    fun saveThatCompletesAfterMoreTypingKeepsTheEditorOpen() =
        runComposeUiTest {
            var saved: (() -> Unit)? = null
            var dismissed = false
            setContent {
                AppTheme {
                    HighlightNoteSheet(
                        fakeHighlight(),
                        { _, done -> saved = done },
                        {},
                        onDismiss = { dismissed = true },
                    )
                }
            }

            onNode(hasSetTextAction()).performTextInput("first")
            onNodeWithText("Save").performClick()
            onNode(hasSetTextAction()).performTextInput(" newer")
            assertNotNull(saved).invoke()
            waitForIdle()

            assertFalse(dismissed)
            onNodeWithText("first newer").assertIsDisplayed()
        }

    @Test
    fun lateSaveCompletionNeverClosesAnotherHighlightsSheet() =
        runComposeUiTest {
            var highlight by mutableStateOf(fakeHighlight(id = "hlt_a"))
            var saved: (() -> Unit)? = null
            var dismissed = false
            setContent {
                AppTheme {
                    HighlightNoteSheet(highlight, { _, done -> saved = done }, {}, onDismiss = { dismissed = true })
                }
            }
            onNode(hasSetTextAction()).performTextInput("first")
            onNodeWithText("Save").performClick()

            highlight = fakeHighlight(id = "hlt_b")
            waitForIdle()
            assertNotNull(saved).invoke()
            waitForIdle()

            assertFalse(dismissed)
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
                        AppTheme {
                            HighlightNoteSheet(fakeHighlight(), onSave = { _, _ -> }, onDeleteNote = {}, onDismiss = {})
                        }
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

    @Test
    fun dismissWithChangesAsksBeforeDiscarding() =
        runComposeUiTest {
            var dismissed = false
            setContent {
                AppTheme {
                    HighlightNoteSheet(fakeHighlight(), { _, _ -> }, {}, onDismiss = { dismissed = true })
                }
            }

            onNode(hasSetTextAction()).performTextInput("A thought")
            onNodeWithContentDescription("Close sheet").performClick()

            onNodeWithText("Discard changes?").assertIsDisplayed()
            assertFalse(dismissed)
            onNodeWithText("Keep editing").performClick()
            onNodeWithText("A thought").assertIsDisplayed()
        }

    @Test
    fun dismissWithoutChangesCloses() =
        runComposeUiTest {
            var dismissed = false
            setContent {
                AppTheme {
                    HighlightNoteSheet(fakeHighlight(), { _, _ -> }, {}, onDismiss = { dismissed = true })
                }
            }

            onNodeWithContentDescription("Close sheet").performClick()
            waitForIdle()

            assertTrue(dismissed)
        }
}

@Composable
private fun NoteEditor(
    highlight: HighlightData,
    onSave: (String) -> Unit,
    onDeleteNote: () -> Unit,
) {
    val opened = highlight.note?.body
    var body by remember { mutableStateOf(opened.orEmpty()) }
    HighlightNoteEditor(highlight, body, { body = it }, opened != null, body != opened.orEmpty(), onSave, onDeleteNote)
}
