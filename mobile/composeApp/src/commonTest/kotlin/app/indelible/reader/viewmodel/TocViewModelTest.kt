package app.indelible.reader.viewmodel

import app.indelible.reader.model.ArticleToc
import app.indelible.reader.model.ArticleTocEntry
import app.indelible.reader.model.ArticleTocStatus
import app.indelible.reader.model.DataPanel
import app.indelible.reader.viewmodel.FakeReaderRepository.Companion.fakeItemDetail
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.launch
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
import kotlin.test.assertTrue

@OptIn(ExperimentalCoroutinesApi::class)
class TocViewModelTest {
    private val testDispatcher = UnconfinedTestDispatcher()
    private lateinit var repository: FakeReaderRepository

    @BeforeTest
    fun setUp() {
        Dispatchers.setMain(testDispatcher)
        repository = FakeReaderRepository()
        repository.getItemResult = Result.success(fakeItemDetail())
    }

    @AfterTest
    fun tearDown() {
        Dispatchers.resetMain()
    }

    private fun entry(
        index: Int,
        words: Int = 200,
    ) = ArticleTocEntry(
        depth = 0,
        id = "ind-toc-$index",
        sourceHeadingIndex = index,
        title = "Section $index",
        wordCount = words,
    )

    private fun toc(
        status: ArticleTocStatus,
        entries: List<ArticleTocEntry> = emptyList(),
    ) = ArticleToc(entries = entries, status = status, truncated = false)

    @Test
    fun pending_polls_until_ready_without_refresh() =
        runTest(testDispatcher) {
            repository.getArticleTocResults =
                listOf(
                    Result.success(toc(ArticleTocStatus.PENDING)),
                    Result.success(toc(ArticleTocStatus.PENDING)),
                    Result.success(toc(ArticleTocStatus.READY, listOf(entry(0), entry(1)))),
                )
            val viewModel = ReaderViewModel("doc_test1", repository)
            advanceUntilIdle()

            val state = assertIs<ReaderUiState.Success>(viewModel.uiState.value)
            assertEquals(TocStatus.READY, state.toc.status)
            assertEquals(2, state.toc.entries.size)
            assertEquals(3, repository.getArticleTocCallCount)
        }

    @Test
    fun none_is_terminal_and_stops_polling() =
        runTest(testDispatcher) {
            repository.getArticleTocResults = listOf(Result.success(toc(ArticleTocStatus.NONE)))
            val viewModel = ReaderViewModel("doc_test1", repository)
            advanceUntilIdle()

            val state = assertIs<ReaderUiState.Success>(viewModel.uiState.value)
            assertEquals(TocStatus.NONE, state.toc.status)
            assertEquals(1, repository.getArticleTocCallCount)
        }

    @Test
    fun exhausted_poll_budget_reports_unavailable() =
        runTest(testDispatcher) {
            repository.getArticleTocResults = listOf(Result.success(toc(ArticleTocStatus.PENDING)))
            val viewModel = ReaderViewModel("doc_test1", repository)
            advanceUntilIdle()

            val state = assertIs<ReaderUiState.Success>(viewModel.uiState.value)
            assertEquals(TocStatus.UNAVAILABLE, state.toc.status)
            assertEquals(15, repository.getArticleTocCallCount)
        }

    @Test
    fun tapping_an_entry_emits_scroll_effect_and_closes_the_panel() =
        runTest(testDispatcher) {
            repository.getArticleTocResults =
                listOf(Result.success(toc(ArticleTocStatus.READY, listOf(entry(3)))))
            val viewModel = ReaderViewModel("doc_test1", repository)
            advanceUntilIdle()
            viewModel.openPanel(DataPanel.CONTENTS)

            val effects = mutableListOf<ReaderEffect>()
            val job = launch { viewModel.effects.collect { effects.add(it) } }
            viewModel.onTocEntryTapped(entry(3))
            advanceUntilIdle()
            job.cancel()

            val scroll = effects.filterIsInstance<ReaderEffect.ScrollToAnchor>().single()
            assertEquals("ind-toc-3", scroll.id)
            assertEquals(3, scroll.fallbackIndex)
            assertEquals(DataPanel.NONE, viewModel.activePanel.value)
        }

    @Test
    fun scroll_progress_updates_the_active_section() =
        runTest(testDispatcher) {
            repository.getArticleTocResults =
                listOf(
                    Result.success(
                        toc(
                            ArticleTocStatus.READY,
                            listOf(entry(0, 100), entry(1, 300), entry(2, 600)),
                        ),
                    ),
                )
            val viewModel = ReaderViewModel("doc_test1", repository)
            advanceUntilIdle()

            viewModel.onContentLoaded()
            viewModel.onScrollRestored()
            viewModel.onScrollProgress(35f)
            val state = assertIs<ReaderUiState.Success>(viewModel.uiState.value)
            assertEquals(1, state.toc.activeIndex)

            viewModel.onScrollProgress(95f)
            val after = assertIs<ReaderUiState.Success>(viewModel.uiState.value)
            assertEquals(2, after.toc.activeIndex)
        }

    @Test
    fun toc_fetch_failures_fall_back_to_unavailable() =
        runTest(testDispatcher) {
            repository.getArticleTocResults = listOf(Result.failure(RuntimeException("network")))
            val viewModel = ReaderViewModel("doc_test1", repository)
            advanceUntilIdle()

            val state = assertIs<ReaderUiState.Success>(viewModel.uiState.value)
            assertEquals(TocStatus.UNAVAILABLE, state.toc.status)
            assertTrue(repository.getArticleTocCallCount >= 1)
        }
}
