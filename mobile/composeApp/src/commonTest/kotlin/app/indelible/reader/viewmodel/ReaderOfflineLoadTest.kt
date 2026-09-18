package app.indelible.reader.viewmodel

import app.indelible.core.offline.DL_DOC
import app.indelible.core.offline.DL_SCOPE
import app.indelible.core.offline.OutboxKind
import app.indelible.reader.repository.NOTE_PATH
import app.indelible.reader.repository.READER_PATH
import app.indelible.reader.repository.ReaderOfflineHarness
import app.indelible.reader.repository.readerOfflineHarness
import io.ktor.client.engine.mock.respond
import io.ktor.http.HttpStatusCode
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runCurrent
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import kotlin.test.AfterTest
import kotlin.test.BeforeTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertIs
import kotlin.test.assertNull
import kotlin.test.assertTrue

// A standard main dispatcher shares runTest's scheduler, so the reader's requests, which run on
// test dispatchers, complete under runCurrent; an unconfined one leaves them unfinished.
@OptIn(ExperimentalCoroutinesApi::class)
class ReaderOfflineLoadTest {
    @BeforeTest
    fun setUp() {
        Dispatchers.setMain(StandardTestDispatcher())
    }

    @AfterTest
    fun tearDown() {
        Dispatchers.resetMain()
    }

    private fun ReaderOfflineHarness.open() = ReaderViewModel(DL_DOC, repository, readingEvents = repository)

    private fun ReaderViewModel.success(): ReaderUiState.Success = assertIs<ReaderUiState.Success>(uiState.value)

    private suspend fun ReaderOfflineHarness.noteRows(): Int =
        store.observeOutbox(DL_SCOPE).first().count { it.kind == OutboxKind.DOCUMENT_NOTE }

    @Test
    fun note_editor_disabled_until_loaded_and_enabled_for_confirmed_absent_note() =
        runTest {
            val release = CompletableDeferred<Unit>()
            val h =
                readerOfflineHarness {
                    route(NOTE_PATH) {
                        release.await()
                        respond("", HttpStatusCode.NotFound)
                    }
                }

            val viewModel = h.open()
            runCurrent()
            assertFalse(viewModel.success().itemNoteLoaded)

            release.complete(Unit)
            runCurrent()
            assertTrue(viewModel.success().itemNoteLoaded)
            assertNull(viewModel.success().itemNote)
        }

    @Test
    fun staleUncachedNoteCannotBeSavedAfterRetryLimit() =
        runTest {
            val h = readerOfflineHarness()
            h.duringFetchOf(NOTE_PATH) { h.acknowledgeNoteEdit() }
            val viewModel = h.open()
            runCurrent()
            val before = h.noteRows()

            viewModel.saveItemNote("would overwrite")
            runCurrent()

            assertFalse(viewModel.success().itemNoteLoaded)
            assertEquals(before, h.noteRows())
        }

    @Test
    fun offline_article_loads_without_polling_to_unavailable() =
        runTest {
            val h = readerOfflineHarness()
            h.keepCopy()
            h.goOffline()
            val readerCalls = h.server.count(READER_PATH)

            val viewModel = h.open()
            advanceUntilIdle()

            val state = viewModel.success()
            assertEquals(ReaderContentStatus.READY, state.contentStatus)
            assertEquals("<p>hello</p>", state.htmlContent)
            assertEquals(TocStatus.NONE, state.toc.status)
            assertEquals("server note", state.itemNote)
            assertTrue(state.itemNoteLoaded)
            assertEquals(1, h.server.count(READER_PATH) - readerCalls)
        }
}
