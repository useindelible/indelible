package app.indelible.reader.viewmodel

import app.indelible.core.offline.OutboxKind
import app.indelible.core.offline.OutboxPayload
import app.indelible.reader.repository.OFFLINE_DOCUMENT_ID
import app.indelible.reader.repository.OFFLINE_SCOPE
import app.indelible.reader.repository.ReaderOutboxHarness
import app.indelible.reader.repository.awaitStore
import app.indelible.reader.repository.readerOutboxHarness
import app.indelible.reader.repository.rowsOf
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import kotlin.test.AfterTest
import kotlin.test.BeforeTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlin.test.assertNotNull
import kotlin.test.assertNull

@OptIn(ExperimentalCoroutinesApi::class)
class ReaderOfflineWritesTest {
    private val testDispatcher = UnconfinedTestDispatcher()
    private lateinit var repository: FakeReaderRepository

    @BeforeTest
    fun setUp() {
        Dispatchers.setMain(testDispatcher)
        repository = FakeReaderRepository()
    }

    @AfterTest
    fun tearDown() {
        Dispatchers.resetMain()
    }

    private suspend fun ReaderOutboxHarness.readingEvents(count: Int): List<OutboxPayload.ReadingEvent> =
        awaitStore {
            store
                .rowsOf(OFFLINE_SCOPE, OutboxKind.READING_EVENT)
                .map { it.payload as OutboxPayload.ReadingEvent }
                .takeIf { it.size >= count }
        }

    @Test
    fun opening_a_reader_enqueues_one_opened_event_without_progress_fields() =
        runTest(testDispatcher) {
            val harness = readerOutboxHarness(backgroundScope)

            ReaderViewModel(OFFLINE_DOCUMENT_ID, repository, readingEvents = harness.repository)
            advanceUntilIdle()

            val event = harness.readingEvents(count = 1).single()
            assertEquals("opened", event.kind)
            assertNull(event.progressBasisPoints)
            assertNull(event.activeMs)
            assertNull(event.positionJson)
        }

    @Test
    fun scroll_progress_enqueues_one_progress_event_in_basis_points() =
        runTest(testDispatcher) {
            val harness = readerOutboxHarness(backgroundScope)
            val viewModel = ReaderViewModel(OFFLINE_DOCUMENT_ID, repository, readingEvents = harness.repository)
            advanceUntilIdle()

            viewModel.onContentLoaded()
            viewModel.onScrollRestored()
            viewModel.onScrollProgress(42.5f)
            advanceUntilIdle()

            val progress = harness.readingEvents(count = 2).single { it.kind == "progress" }
            assertEquals(4250, progress.progressBasisPoints)
        }

    @Test
    fun every_event_of_one_reader_shares_the_session_id() =
        runTest(testDispatcher) {
            val harness = readerOutboxHarness(backgroundScope)
            val viewModel = ReaderViewModel(OFFLINE_DOCUMENT_ID, repository, readingEvents = harness.repository)
            advanceUntilIdle()

            viewModel.onContentLoaded()
            viewModel.onScrollRestored()
            viewModel.onScrollProgress(10f)
            advanceUntilIdle()

            val events = harness.readingEvents(count = 2)
            assertEquals(2, events.size)
            val sessionId = assertNotNull(events.first().sessionId)
            assertEquals(listOf(sessionId, sessionId), events.map { it.sessionId })
        }

    @Test
    fun a_writer_that_cannot_record_the_open_still_lets_the_document_load() =
        runTest(testDispatcher) {
            val viewModel =
                ReaderViewModel(
                    OFFLINE_DOCUMENT_ID,
                    repository,
                    readingEvents = RecordingReadingEventWriter(failWith = IllegalStateException("no store")),
                )
            advanceUntilIdle()

            val state = assertIs<ReaderUiState.Success>(viewModel.uiState.value)
            assertEquals("<p>Hello world</p>", state.htmlContent)
        }
}
