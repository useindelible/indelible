package app.indelible.reader.viewmodel

import app.indelible.core.i18n.UiMessage
import app.indelible.reader.model.DataPanel
import app.indelible.reader.model.ReaderContentMode
import app.indelible.reader.model.ReaderPreferences
import app.indelible.reader.model.Typeface
import app.indelible.reader.viewmodel.FakeReaderRepository.Companion.fakeItemDetail
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.reader_error_load
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.toList
import kotlinx.coroutines.launch
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import kotlinx.datetime.Instant
import kotlin.test.AfterTest
import kotlin.test.BeforeTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertIs
import kotlin.test.assertNull
import kotlin.test.assertTrue

@OptIn(ExperimentalCoroutinesApi::class)
class ReaderViewModelTest {
    private val testDispatcher = UnconfinedTestDispatcher()
    private lateinit var repository: FakeReaderRepository
    private lateinit var events: RecordingReadingEventWriter
    private lateinit var viewModel: ReaderViewModel

    @BeforeTest
    fun setUp() {
        Dispatchers.setMain(testDispatcher)
        repository = FakeReaderRepository()
        events = RecordingReadingEventWriter()
    }

    @AfterTest
    fun tearDown() {
        Dispatchers.resetMain()
    }

    @Test
    fun load_item_transitions_to_success() =
        runTest(testDispatcher) {
            repository.getItemResult = Result.success(fakeItemDetail())
            viewModel = ReaderViewModel("doc_test1", repository, readingEvents = events)
            advanceUntilIdle()

            val state = assertIs<ReaderUiState.Success>(viewModel.uiState.value)
            assertEquals("doc_test1", state.item.id)
            assertEquals(ReaderContentMode.HTML, state.contentMode)
        }

    @Test
    fun load_item_failure_transitions_to_error() =
        runTest(testDispatcher) {
            repository.getItemResult = Result.failure(RuntimeException("Not found"))
            viewModel = ReaderViewModel("doc_test1", repository, readingEvents = events)
            advanceUntilIdle()

            val state = assertIs<ReaderUiState.Error>(viewModel.uiState.value)
            assertEquals(UiMessage(Res.string.reader_error_load), state.message)
        }

    @Test
    fun html_content_loaded_on_success() =
        runTest(testDispatcher) {
            repository.getItemResult = Result.success(fakeItemDetail())
            repository.fetchHtmlResult = Result.success("<p>Article content</p>")
            viewModel = ReaderViewModel("doc_test1", repository, readingEvents = events)
            advanceUntilIdle()

            val state = assertIs<ReaderUiState.Success>(viewModel.uiState.value)
            assertEquals("<p>Article content</p>", state.htmlContent)
        }

    @Test
    fun save_to_library_flips_saved_state_and_saves_by_url() =
        runTest(testDispatcher) {
            repository.getItemResults =
                listOf(
                    Result.success(fakeItemDetail(saved = false)),
                    Result.success(fakeItemDetail(saved = true)),
                )
            viewModel = ReaderViewModel("doc_test1", repository, readingEvents = events)
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
            viewModel = ReaderViewModel("doc_test1", repository, readingEvents = events)
            advanceUntilIdle()

            val state = assertIs<ReaderUiState.Success>(viewModel.uiState.value)
            assertEquals(ReaderContentMode.PDF_COMING_SOON, state.contentMode)
            assertNull(state.htmlContent)
        }

    @Test
    fun update_preferences_updates_state() =
        runTest(testDispatcher) {
            repository.getItemResult = Result.success(fakeItemDetail())
            viewModel = ReaderViewModel("doc_test1", repository, readingEvents = events)
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
            viewModel = ReaderViewModel("doc_test1", repository, readingEvents = events)
            advanceUntilIdle()

            viewModel.onContentLoaded()
            viewModel.onScrollRestored()
            viewModel.onScrollProgress(42.5f)
            advanceUntilIdle()

            val state = assertIs<ReaderUiState.Success>(viewModel.uiState.value)
            assertEquals(42.5f, state.progress)
            assertEquals(42.5f, events.progressPercents.last())
        }

    @Test
    fun initial_scroll_event_does_not_overwrite_saved_progress() =
        runTest(testDispatcher) {
            repository.getItemResult =
                Result.success(
                    fakeItemDetail(progressPercent = 60f),
                )
            viewModel = ReaderViewModel("doc_test1", repository, readingEvents = events)
            advanceUntilIdle()

            viewModel.onScrollProgress(0f)

            val state = assertIs<ReaderUiState.Success>(viewModel.uiState.value)
            assertEquals(60f, state.progress)
            assertTrue(events.progressPercents.isEmpty())
        }

    @Test
    fun saved_progress_restored_on_load() =
        runTest(testDispatcher) {
            repository.getItemResult =
                Result.success(
                    fakeItemDetail(progressPercent = 60f),
                )
            viewModel = ReaderViewModel("doc_test1", repository, readingEvents = events)
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
            viewModel = ReaderViewModel("doc_test1", repository, readingEvents = events)
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
            viewModel = ReaderViewModel("doc_test1", repository, readingEvents = events)
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
            viewModel = ReaderViewModel("doc_test1", repository, readingEvents = events)
            advanceUntilIdle()

            val state = assertIs<ReaderUiState.Success>(viewModel.uiState.value)
            assertEquals(0f, state.progress)
        }

    @Test
    fun move_to_triage_calls_repository() =
        runTest(testDispatcher) {
            repository.getItemResult = Result.success(fakeItemDetail())
            repository.triageItemResult = Result.success(Unit)
            viewModel = ReaderViewModel("doc_test1", repository, readingEvents = events)
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
            viewModel = ReaderViewModel("doc_test1", repository, readingEvents = events)
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
            viewModel = ReaderViewModel("doc_test1", repository, readingEvents = events)
            advanceUntilIdle()

            val effects = mutableListOf<ReaderEffect>()
            val job = launch { viewModel.effects.toList(effects) }

            viewModel.navigateBack()
            advanceUntilIdle()
            job.cancel()

            assertTrue(effects.any { it is ReaderEffect.NavigateBack })
        }
}
