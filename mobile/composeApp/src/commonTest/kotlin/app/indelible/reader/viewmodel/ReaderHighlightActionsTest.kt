package app.indelible.reader.viewmodel

import app.indelible.reader.model.HighlightColor
import app.indelible.reader.model.HighlightRange
import app.indelible.reader.model.HighlightTarget
import app.indelible.reader.model.PdfHighlightTarget
import app.indelible.reader.model.PdfRect
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.runCurrent
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

    @Test
    fun pdfTargetIsBuiltBeforeTheWriteStarts() =
        runTest {
            val repository = FakeReaderRepository()
            val actions =
                ReaderHighlightActions(
                    documentId = "doc_1",
                    repository = repository,
                    scope = this,
                    uiState = MutableStateFlow(ReaderUiState.Loading),
                    effects = MutableSharedFlow(),
                )
            val target = PdfHighlightTarget(3, 0.1, 0.2, 0.5, 0.04, "quick", listOf(PdfRect(0.1, 0.2, 0.5, 0.02)))

            actions.createPdfHighlight(HighlightColor.BLUE, target, "quick")
            assertEquals(emptyList<HighlightTarget>(), repository.createdTargets)
            runCurrent()

            assertEquals(listOf<HighlightTarget>(target), repository.createdTargets)
            assertEquals(emptyList<HighlightRange>(), repository.createdRanges)
        }
}
