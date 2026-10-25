package app.indelible.reader.ui

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.SnackbarHostState
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalClipboardManager
import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.runComposeUiTest
import app.indelible.reader.model.HighlightColor
import app.indelible.reader.model.PdfHighlightTarget
import app.indelible.reader.model.TagData
import app.indelible.reader.pdf.FakePdfDocument
import app.indelible.reader.pdf.NormalizedRect
import app.indelible.reader.pdf.PdfPageSize
import app.indelible.reader.pdf.PdfSelection
import app.indelible.reader.viewmodel.FakeReaderRepository
import app.indelible.reader.viewmodel.FakeReaderRepository.Companion.fakeItemDetail
import app.indelible.reader.viewmodel.ReaderViewModel
import app.indelible.reader.viewmodel.RecordingReadingEventWriter
import app.indelible.ui.theme.AppTheme
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.setMain
import kotlin.test.AfterTest
import kotlin.test.BeforeTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs

@OptIn(ExperimentalTestApi::class, ExperimentalCoroutinesApi::class)
class HighlightToolbarOverlayTest {
    private val repository = FakeReaderRepository()
    private val document = FakePdfDocument(List(3) { PdfPageSize(612.0, 792.0) })
    private val selection = PdfSelection(2, "quick brown", 4, 15, listOf(NormalizedRect(0.1, 0.2, 0.5, 0.02)))

    @BeforeTest
    fun setUp() {
        Dispatchers.setMain(UnconfinedTestDispatcher())
        repository.getItemResult = Result.success(fakeItemDetail(itemType = "pdf"))
        repository.listTagsResult = Result.success(listOf(TagData(id = "tag_1", name = "research")))
    }

    @AfterTest
    fun tearDown() {
        Dispatchers.resetMain()
    }

    @Test
    fun tagOnAPdfSelectionCreatesThePdfHighlightThenOpensTheTagSheet() =
        runComposeUiTest {
            val viewModel =
                ReaderViewModel("doc_test1", repository, readingEvents = RecordingReadingEventWriter()) { document }
            val anchor = SelectionRect(0f, 64f, 10f, 10f)
            var opened: String? = null
            var tags: List<TagData> = emptyList()
            setContent {
                AppTheme {
                    Box(Modifier.fillMaxSize()) {
                        HighlightToolbarOverlay(
                            sel = SelectedTextInfo("quick brown", 0, 0, anchor, pdf = selection),
                            defaultHighlightColor = HighlightColor.YELLOW,
                            snackbarHostState = SnackbarHostState(),
                            coroutineScope = rememberCoroutineScope(),
                            clipboardManager = LocalClipboardManager.current,
                            viewModel = viewModel,
                            onSelectedTextChanged = {},
                            onAvailableTagsChanged = { tags = it },
                            onTagSheetHighlightIdChanged = { opened = it },
                            onNoteSheetHighlightIdChanged = {},
                        )
                    }
                }
            }

            onNodeWithText("Tag").performClick()
            waitForIdle()

            val target = assertIs<PdfHighlightTarget>(repository.createdTargets.single())
            assertEquals(3, target.page)
            assertEquals("hlt_test1", opened)
            assertEquals(listOf("research"), tags.map { it.name })
        }
}
