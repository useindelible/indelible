package app.indelible.search.viewmodel

import app.indelible.search.model.PaginatedSearchResults
import app.indelible.search.model.RecentSearch
import app.indelible.search.model.SearchResult
import app.indelible.search.model.SearchSuggestion
import app.indelible.search.repository.SearchRepository
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import kotlinx.datetime.Instant
import kotlin.test.AfterTest
import kotlin.test.BeforeTest
import kotlin.test.Test
import kotlin.test.assertEquals

private val WHEN = Instant.fromEpochMilliseconds(0)

private fun result(
    documentId: String?,
    title: String,
    deliveryId: String? = null,
) = SearchResult(
    contentType = "article",
    deliveryId = deliveryId,
    documentId = documentId,
    resultKind = if (documentId != null) "document" else "feed_preview",
    savedAt = WHEN,
    score = 1.0,
    snippet = "snippet",
    title = title,
    updatedAt = WHEN,
)

private class FakeSearchRepository(
    private val pages: List<PaginatedSearchResults>,
) : SearchRepository {
    private var page = 0

    override suspend fun search(
        query: String,
        cursor: String?,
        limit: Int,
    ): Result<PaginatedSearchResults> {
        page = if (cursor == null) 0 else (page + 1).coerceAtMost(pages.lastIndex)
        return Result.success(pages[page])
    }

    override suspend fun suggestions(
        query: String,
        limit: Int,
    ): Result<List<SearchSuggestion>> = Result.success(emptyList())

    override suspend fun listRecentSearches(limit: Int): Result<List<RecentSearch>> = Result.success(emptyList())

    override suspend fun deleteRecentSearch(id: String): Result<Unit> = Result.success(Unit)

    override suspend fun clearRecentSearches(): Result<Unit> = Result.success(Unit)
}

@OptIn(ExperimentalCoroutinesApi::class)
class SearchViewModelTest {
    @BeforeTest
    fun setUp() {
        Dispatchers.setMain(StandardTestDispatcher())
    }

    @AfterTest
    fun tearDown() {
        Dispatchers.resetMain()
    }

    @Test
    fun aDocumentRepeatedInOnePageIsListedOnce() =
        runTest {
            val page =
                PaginatedSearchResults(
                    hasMore = false,
                    query = "design",
                    results =
                        listOf(
                            result("doc_1", "System Design Interview"),
                            result("doc_1", "System Design Interview"),
                            result("doc_2", "Another"),
                        ),
                )
            val viewModel = SearchViewModel(FakeSearchRepository(listOf(page)))

            viewModel.onQueryChange("design")
            advanceUntilIdle()

            assertEquals(listOf("doc_1", "doc_2"), viewModel.state.value.results.map { it.documentId })
        }

    @Test
    fun aDocumentRepeatedOnTheNextPageIsListedOnce() =
        runTest {
            val first =
                PaginatedSearchResults(
                    hasMore = true,
                    nextCursor = "cursor",
                    query = "design",
                    results = listOf(result("doc_1", "System Design Interview")),
                )
            val second =
                PaginatedSearchResults(
                    hasMore = false,
                    query = "design",
                    results = listOf(result("doc_1", "System Design Interview"), result("doc_3", "Third")),
                )
            val viewModel = SearchViewModel(FakeSearchRepository(listOf(first, second)))

            viewModel.onQueryChange("design")
            advanceUntilIdle()
            viewModel.loadNextPage()
            advanceUntilIdle()

            assertEquals(listOf("doc_1", "doc_3"), viewModel.state.value.results.map { it.documentId })
        }

    @Test
    fun feedPreviewsWithoutDocumentIdsAreKept() =
        runTest {
            val page =
                PaginatedSearchResults(
                    hasMore = false,
                    query = "design",
                    results =
                        listOf(
                            result(documentId = null, title = "One", deliveryId = "dlv_1"),
                            result(documentId = null, title = "Two", deliveryId = "dlv_2"),
                        ),
                )
            val viewModel = SearchViewModel(FakeSearchRepository(listOf(page)))

            viewModel.onQueryChange("design")
            advanceUntilIdle()

            assertEquals(listOf("dlv_1", "dlv_2"), viewModel.state.value.results.map { it.deliveryId })
        }
}
