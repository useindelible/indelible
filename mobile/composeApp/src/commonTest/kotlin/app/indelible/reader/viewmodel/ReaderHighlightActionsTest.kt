package app.indelible.reader.viewmodel

import app.indelible.reader.model.HighlightColor
import app.indelible.reader.model.HighlightRange
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals

class ReaderHighlightActionsTest {
    @Test
    fun chapterIsCapturedWhenTheHighlightIsRequested() =
        runTest {
            val repository = FakeReaderRepository()
            var chapter = "ch2"
            val actions =
                ReaderHighlightActions(
                    documentId = "doc_1",
                    repository = repository,
                    scope = this,
                    uiState = MutableStateFlow(ReaderUiState.Loading),
                    effects = MutableSharedFlow(),
                    chapterKey = { chapter },
                )

            actions.createHighlight(HighlightColor.YELLOW, "quoted", startOffset = 3, endOffset = 9)
            chapter = "ch3"
            advanceUntilIdle()

            assertEquals(listOf(HighlightRange(3, 9, chapter = "ch2")), repository.createdRanges)
        }
}
