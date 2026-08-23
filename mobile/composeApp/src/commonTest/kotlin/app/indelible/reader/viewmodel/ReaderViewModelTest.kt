package app.indelible.reader.viewmodel

import app.indelible.core.i18n.UiMessage
import app.indelible.reader.model.DataPanel
import app.indelible.reader.model.HighlightColor
import app.indelible.reader.model.ReaderContentMode
import app.indelible.reader.model.ReaderPreferences
import app.indelible.reader.model.ReaderReprocessResult
import app.indelible.reader.model.Typeface
import app.indelible.reader.viewmodel.FakeReaderRepository.Companion.fakeHighlight
import app.indelible.reader.viewmodel.FakeReaderRepository.Companion.fakeItemDetail
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.reader_error_load
import indelible.composeapp.generated.resources.reader_error_retry_content
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.toList
import kotlinx.coroutines.launch
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runCurrent
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import kotlinx.datetime.Instant
import kotlin.test.AfterTest
import kotlin.test.BeforeTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertIs
import kotlin.test.assertNotNull
import kotlin.test.assertNull
import kotlin.test.assertTrue

@OptIn(ExperimentalCoroutinesApi::class)
class ReaderViewModelTest {
    private val testDispatcher = UnconfinedTestDispatcher()
    private lateinit var repository: FakeReaderRepository
    private lateinit var viewModel: ReaderViewModel

    @BeforeTest
    fun setUp() {
        Dispatchers.setMain(testDispatcher)
        repository = FakeReaderRepository()
    }

    @AfterTest
    fun tearDown() {
        Dispatchers.resetMain()
    }

    @Test
    fun load_item_transitions_to_success() =
        runTest(testDispatcher) {
            repository.getItemResult = Result.success(fakeItemDetail())
            viewModel = ReaderViewModel("doc_test1", repository)
            advanceUntilIdle()

            val state = assertIs<ReaderUiState.Success>(viewModel.uiState.value)
            assertEquals("doc_test1", state.item.id)
            assertEquals(ReaderContentMode.HTML, state.contentMode)
        }

    @Test
    fun load_item_failure_transitions_to_error() =
        runTest(testDispatcher) {
            repository.getItemResult = Result.failure(RuntimeException("Not found"))
            viewModel = ReaderViewModel("doc_test1", repository)
            advanceUntilIdle()

            val state = assertIs<ReaderUiState.Error>(viewModel.uiState.value)
            assertEquals(UiMessage(Res.string.reader_error_load), state.message)
        }

    @Test
    fun html_content_loaded_on_success() =
        runTest(testDispatcher) {
            repository.getItemResult = Result.success(fakeItemDetail())
            repository.fetchHtmlResult = Result.success("<p>Article content</p>")
            viewModel = ReaderViewModel("doc_test1", repository)
            advanceUntilIdle()

            val state = assertIs<ReaderUiState.Success>(viewModel.uiState.value)
            assertEquals("<p>Article content</p>", state.htmlContent)
        }

    @Test
    fun polls_until_readable_then_loads_html() =
        runTest(testDispatcher) {
            repository.getItemResults =
                listOf(
                    Result.success(fakeItemDetail(readableReady = false)),
                    Result.success(fakeItemDetail(readableReady = true)),
                )
            repository.fetchHtmlResult = Result.success("<p>Prepared content</p>")
            viewModel = ReaderViewModel("doc_test1", repository)
            advanceUntilIdle()

            val state = assertIs<ReaderUiState.Success>(viewModel.uiState.value)
            assertEquals(ReaderContentStatus.READY, state.contentStatus)
            assertEquals("<p>Prepared content</p>", state.htmlContent)
        }

    @Test
    fun content_unavailable_after_polls_exhausted() =
        runTest(testDispatcher) {
            repository.getItemResult = Result.success(fakeItemDetail(readableReady = false))
            viewModel = ReaderViewModel("doc_test1", repository)
            advanceUntilIdle()

            val state = assertIs<ReaderUiState.Success>(viewModel.uiState.value)
            assertEquals(ReaderContentStatus.UNAVAILABLE, state.contentStatus)
            assertNull(state.htmlContent)
        }

    @Test
    fun retry_reloads_content_after_unavailable() =
        runTest(testDispatcher) {
            repository.getItemResult = Result.success(fakeItemDetail(readableReady = false))
            viewModel = ReaderViewModel("doc_test1", repository)
            advanceUntilIdle()
            assertEquals(
                ReaderContentStatus.UNAVAILABLE,
                assertIs<ReaderUiState.Success>(viewModel.uiState.value).contentStatus,
            )

            repository.getItemResult = Result.success(fakeItemDetail(readableReady = true))
            repository.fetchHtmlResult = Result.success("<p>Now ready</p>")
            viewModel.retryLoadContent()
            advanceUntilIdle()

            val state = assertIs<ReaderUiState.Success>(viewModel.uiState.value)
            assertEquals(ReaderContentStatus.READY, state.contentStatus)
            assertEquals("<p>Now ready</p>", state.htmlContent)
        }

    @Test
    fun retry_reprocesses_document_before_polling_or_fetching_content() =
        runTest(testDispatcher) {
            repository.getItemResult = Result.success(fakeItemDetail(readableReady = false))
            viewModel = ReaderViewModel("doc_test1", repository)
            advanceUntilIdle()
            assertEquals(
                ReaderContentStatus.UNAVAILABLE,
                assertIs<ReaderUiState.Success>(viewModel.uiState.value).contentStatus,
            )

            repository.contentCallLog.clear()
            repository.getItemResult = Result.success(fakeItemDetail(readableReady = true))
            repository.fetchHtmlResult = Result.success("<p>Now ready</p>")

            viewModel.retryLoadContent()
            advanceUntilIdle()

            val reprocessIndex = repository.contentCallLog.indexOf("reprocessDocument:doc_test1")
            val pollIndex = repository.contentCallLog.indexOf("getItem:doc_test1")
            val fetchIndex = repository.contentCallLog.indexOf("fetchReadableHtml:doc_test1")
            assertTrue(reprocessIndex >= 0, repository.contentCallLog.toString())
            assertTrue(pollIndex > reprocessIndex, repository.contentCallLog.toString())
            assertTrue(fetchIndex > reprocessIndex, repository.contentCallLog.toString())
        }

    @Test
    fun failed_retry_reprocess_keeps_content_unavailable_and_emits_error() =
        runTest(testDispatcher) {
            repository.getItemResult = Result.success(fakeItemDetail(readableReady = false))
            viewModel = ReaderViewModel("doc_test1", repository)
            advanceUntilIdle()
            assertEquals(
                ReaderContentStatus.UNAVAILABLE,
                assertIs<ReaderUiState.Success>(viewModel.uiState.value).contentStatus,
            )

            val effects = mutableListOf<ReaderEffect>()
            val job = launch { viewModel.effects.toList(effects) }
            repository.contentCallLog.clear()
            repository.reprocessDocumentResult = Result.failure(RuntimeException("Queue failed"))
            repository.getItemResult = Result.success(fakeItemDetail(readableReady = true))
            repository.fetchHtmlResult = Result.success("<p>Should not load</p>")

            viewModel.retryLoadContent()
            advanceUntilIdle()
            job.cancel()

            val state = assertIs<ReaderUiState.Success>(viewModel.uiState.value)
            assertEquals(ReaderContentStatus.UNAVAILABLE, state.contentStatus)
            assertNull(state.htmlContent)
            assertTrue(
                effects
                    .filterIsInstance<ReaderEffect.ShowSnackbar>()
                    .any { it.message == UiMessage(Res.string.reader_error_retry_content) },
                effects.toString(),
            )
            assertEquals(listOf("reprocessDocument:doc_test1"), repository.contentCallLog)
        }

    @Test
    fun retry_surfaces_server_cooldown_while_polling() =
        runTest(testDispatcher) {
            repository.getItemResult = Result.success(fakeItemDetail(readableReady = false))
            viewModel = ReaderViewModel("doc_test1", repository)
            advanceUntilIdle()
            repository.reprocessDocumentResult =
                Result.success(ReaderReprocessResult(queued = false, retryAfterSeconds = 45))

            viewModel.retryLoadContent()
            runCurrent()

            val cooling = assertIs<ReaderUiState.Success>(viewModel.uiState.value)
            assertEquals(ReaderRetryStatus.COOLDOWN, cooling.retryStatus)
            assertEquals(45L, cooling.retryAfterSeconds)

            advanceUntilIdle()
            val exhausted = assertIs<ReaderUiState.Success>(viewModel.uiState.value)
            assertEquals(ReaderRetryStatus.IDLE, exhausted.retryStatus)
            assertEquals(ReaderContentStatus.UNAVAILABLE, exhausted.contentStatus)
        }

    @Test
    fun save_to_library_flips_saved_state_and_saves_by_url() =
        runTest(testDispatcher) {
            repository.getItemResults =
                listOf(
                    Result.success(fakeItemDetail(saved = false)),
                    Result.success(fakeItemDetail(saved = true)),
                )
            viewModel = ReaderViewModel("doc_test1", repository)
            advanceUntilIdle()
            assertFalse(assertIs<ReaderUiState.Success>(viewModel.uiState.value).item.saved)

            viewModel.saveToLibrary()
            advanceUntilIdle()

            val state = assertIs<ReaderUiState.Success>(viewModel.uiState.value)
            assertTrue(state.item.saved)
            assertEquals("https://example.com/article", repository.lastSavedUrl)
        }

    @Test
    fun pdf_item_uses_pdf_coming_soon_mode_without_fetching_html() =
        runTest(testDispatcher) {
            repository.getItemResult = Result.success(fakeItemDetail(itemType = "pdf"))
            viewModel = ReaderViewModel("doc_test1", repository)
            advanceUntilIdle()

            val state = assertIs<ReaderUiState.Success>(viewModel.uiState.value)
            assertEquals(ReaderContentMode.PDF_COMING_SOON, state.contentMode)
            assertNull(state.htmlContent)
        }

    @Test
    fun book_item_uses_epub_coming_soon_mode_without_fetching_html() =
        runTest(testDispatcher) {
            repository.getItemResult = Result.success(fakeItemDetail(itemType = "book"))
            viewModel = ReaderViewModel("doc_test1", repository)
            advanceUntilIdle()

            val state = assertIs<ReaderUiState.Success>(viewModel.uiState.value)
            assertEquals(ReaderContentMode.EPUB_COMING_SOON, state.contentMode)
            assertNull(state.htmlContent)
        }

    @Test
    fun update_preferences_updates_state() =
        runTest(testDispatcher) {
            repository.getItemResult = Result.success(fakeItemDetail())
            viewModel = ReaderViewModel("doc_test1", repository)
            advanceUntilIdle()

            val newPrefs = ReaderPreferences(typeface = Typeface.SERIF)
            viewModel.updatePreferences(newPrefs)

            val state = assertIs<ReaderUiState.Success>(viewModel.uiState.value)
            assertEquals(Typeface.SERIF, state.preferences.typeface)
        }

    @Test
    fun scroll_progress_updates_state() =
        runTest(testDispatcher) {
            repository.getItemResult = Result.success(fakeItemDetail())
            viewModel = ReaderViewModel("doc_test1", repository)
            advanceUntilIdle()

            viewModel.onContentLoaded()
            viewModel.onScrollRestored()
            viewModel.onScrollProgress(42.5f)
            advanceUntilIdle()

            val state = assertIs<ReaderUiState.Success>(viewModel.uiState.value)
            assertEquals(42.5f, state.progress)
            assertEquals(42.5f, repository.lastProgressPercent)
        }

    @Test
    fun initial_scroll_event_does_not_overwrite_saved_progress() =
        runTest(testDispatcher) {
            repository.getItemResult =
                Result.success(
                    fakeItemDetail(progressPercent = 60f),
                )
            viewModel = ReaderViewModel("doc_test1", repository)
            advanceUntilIdle()

            viewModel.onScrollProgress(0f)

            val state = assertIs<ReaderUiState.Success>(viewModel.uiState.value)
            assertEquals(60f, state.progress)
            assertNull(repository.lastProgressPercent)
        }

    @Test
    fun saved_progress_restored_on_load() =
        runTest(testDispatcher) {
            repository.getItemResult =
                Result.success(
                    fakeItemDetail(progressPercent = 60f),
                )
            viewModel = ReaderViewModel("doc_test1", repository)
            advanceUntilIdle()

            val state = assertIs<ReaderUiState.Success>(viewModel.uiState.value)
            assertEquals(60f, state.progress)
        }

    @Test
    fun content_loaded_emits_scroll_to_percent_when_has_progress() =
        runTest(testDispatcher) {
            repository.getItemResult =
                Result.success(
                    fakeItemDetail(progressPercent = 60f),
                )
            viewModel = ReaderViewModel("doc_test1", repository)
            advanceUntilIdle()

            val effects = mutableListOf<ReaderEffect>()
            val job = launch { viewModel.effects.toList(effects) }

            viewModel.onContentLoaded()
            advanceUntilIdle()
            job.cancel()

            assertTrue(effects.any { it is ReaderEffect.ScrollToPercent })
            val scrollEffect = effects.filterIsInstance<ReaderEffect.ScrollToPercent>().first()
            assertEquals(60f, scrollEffect.percent)
        }

    @Test
    fun finished_item_starts_at_beginning() =
        runTest(testDispatcher) {
            repository.getItemResult =
                Result.success(
                    fakeItemDetail(
                        progressPercent = 100f,
                        maxProgressPercent = 100f,
                        finishedAt = Instant.parse("2026-07-28T12:00:00Z"),
                    ),
                )
            viewModel = ReaderViewModel("doc_test1", repository)
            advanceUntilIdle()

            val state = assertIs<ReaderUiState.Success>(viewModel.uiState.value)
            assertEquals(0f, state.progress)

            val effects = mutableListOf<ReaderEffect>()
            val job = launch { viewModel.effects.toList(effects) }
            viewModel.onContentLoaded()
            advanceUntilIdle()
            job.cancel()

            val scrollEffect = effects.filterIsInstance<ReaderEffect.ScrollToPercent>().first()
            assertEquals(0f, scrollEffect.percent)
        }

    @Test
    fun one_hundred_percent_item_without_completion_metadata_starts_at_beginning() =
        runTest(testDispatcher) {
            repository.getItemResult =
                Result.success(
                    fakeItemDetail(progressPercent = 100f),
                )
            viewModel = ReaderViewModel("doc_test1", repository)
            advanceUntilIdle()

            val state = assertIs<ReaderUiState.Success>(viewModel.uiState.value)
            assertEquals(0f, state.progress)
        }

    @Test
    fun create_highlight_adds_to_state() =
        runTest(testDispatcher) {
            val highlight = fakeHighlight(id = "hlt_new", color = "Green")
            repository.getItemResult = Result.success(fakeItemDetail())
            repository.createHighlightResult = Result.success(highlight)
            viewModel = ReaderViewModel("doc_test1", repository)
            advanceUntilIdle()

            viewModel.createHighlight(HighlightColor.GREEN, "text", 0, 10)
            advanceUntilIdle()

            val state = assertIs<ReaderUiState.Success>(viewModel.uiState.value)
            assertTrue(state.highlights.any { it.id == "hlt_new" })
            assertEquals(1, repository.createHighlightCallCount)
        }

    @Test
    fun delete_highlight_removes_from_state() =
        runTest(testDispatcher) {
            val highlight = fakeHighlight(id = "hlt_remove")
            repository.getItemResult = Result.success(fakeItemDetail())
            repository.listHighlightsResult = Result.success(listOf(highlight))
            viewModel = ReaderViewModel("doc_test1", repository)
            advanceUntilIdle()

            viewModel.deleteHighlight("hlt_remove")
            advanceUntilIdle()

            val state = assertIs<ReaderUiState.Success>(viewModel.uiState.value)
            assertTrue(state.highlights.none { it.id == "hlt_remove" })
            assertEquals("hlt_remove", repository.lastDeletedHighlightId)
        }

    @Test
    fun delete_highlight_rollback_on_failure() =
        runTest(testDispatcher) {
            val highlight = fakeHighlight(id = "hlt_fail")
            repository.getItemResult = Result.success(fakeItemDetail())
            repository.listHighlightsResult = Result.success(listOf(highlight))
            repository.deleteHighlightResult = Result.failure(RuntimeException("Server error"))
            viewModel = ReaderViewModel("doc_test1", repository)
            advanceUntilIdle()

            viewModel.deleteHighlight("hlt_fail")
            advanceUntilIdle()

            val state = assertIs<ReaderUiState.Success>(viewModel.uiState.value)
            assertTrue(state.highlights.any { it.id == "hlt_fail" })
        }

    @Test
    fun update_highlight_color_updates_state() =
        runTest(testDispatcher) {
            val highlight = fakeHighlight(id = "hlt_color", color = "Yellow")
            val updated = highlight.copy(color = "Blue")
            repository.getItemResult = Result.success(fakeItemDetail())
            repository.listHighlightsResult = Result.success(listOf(highlight))
            repository.updateHighlightColorResult = Result.success(updated)
            viewModel = ReaderViewModel("doc_test1", repository)
            advanceUntilIdle()

            viewModel.updateHighlightColor("hlt_color", HighlightColor.BLUE)
            advanceUntilIdle()

            val state = assertIs<ReaderUiState.Success>(viewModel.uiState.value)
            val found = state.highlights.find { it.id == "hlt_color" }
            assertNotNull(found)
            assertEquals("Blue", found.color)
        }

    @Test
    fun move_to_triage_calls_repository() =
        runTest(testDispatcher) {
            repository.getItemResult = Result.success(fakeItemDetail())
            repository.triageItemResult = Result.success(Unit)
            viewModel = ReaderViewModel("doc_test1", repository)
            advanceUntilIdle()

            viewModel.moveToTriage("archive")
            advanceUntilIdle()

            assertEquals("archive", repository.lastTriagedState)
            assertEquals(DataPanel.NONE, viewModel.activePanel.value)
        }

    @Test
    fun move_to_triage_rolls_back_on_failure() =
        runTest(testDispatcher) {
            repository.getItemResult = Result.success(fakeItemDetail())
            repository.triageItemResult = Result.failure(RuntimeException("Server error"))
            viewModel = ReaderViewModel("doc_test1", repository)
            advanceUntilIdle()

            viewModel.moveToTriage("archive")
            advanceUntilIdle()

            val state = assertIs<ReaderUiState.Success>(viewModel.uiState.value)
            assertEquals("inbox", state.item.triageState)
        }

    @Test
    fun navigate_back_emits_effect() =
        runTest(testDispatcher) {
            repository.getItemResult = Result.success(fakeItemDetail())
            viewModel = ReaderViewModel("doc_test1", repository)
            advanceUntilIdle()

            val effects = mutableListOf<ReaderEffect>()
            val job = launch { viewModel.effects.toList(effects) }

            viewModel.navigateBack()
            advanceUntilIdle()
            job.cancel()

            assertTrue(effects.any { it is ReaderEffect.NavigateBack })
        }

    @Test
    fun highlights_loaded_on_init() =
        runTest(testDispatcher) {
            val highlights =
                listOf(
                    fakeHighlight(id = "hlt_1"),
                    fakeHighlight(id = "hlt_2"),
                )
            repository.getItemResult = Result.success(fakeItemDetail())
            repository.listHighlightsResult = Result.success(highlights)
            viewModel = ReaderViewModel("doc_test1", repository)
            advanceUntilIdle()

            val state = assertIs<ReaderUiState.Success>(viewModel.uiState.value)
            assertEquals(2, state.highlights.size)
        }

    @Test
    fun upsert_note_updates_highlight_in_state() =
        runTest(testDispatcher) {
            val highlight = fakeHighlight(id = "hlt_note")
            val note = FakeReaderRepository.fakeNote(highlightId = "hlt_note")
            repository.getItemResult = Result.success(fakeItemDetail())
            repository.listHighlightsResult = Result.success(listOf(highlight))
            repository.upsertNoteResult = Result.success(note)
            viewModel = ReaderViewModel("doc_test1", repository)
            advanceUntilIdle()

            assertNull(
                (viewModel.uiState.value as ReaderUiState.Success)
                    .highlights
                    .first()
                    .note,
            )

            viewModel.upsertHighlightNote("hlt_note", "my note")
            advanceUntilIdle()

            val state = assertIs<ReaderUiState.Success>(viewModel.uiState.value)
            val found = state.highlights.find { it.id == "hlt_note" }
            assertNotNull(found?.note)
            assertEquals("test note", found.note?.body)
        }
}
