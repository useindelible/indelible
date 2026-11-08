package app.indelible.search.viewmodel

import app.indelible.search.model.PaginatedSearchResults
import app.indelible.search.model.RecentSearch
import app.indelible.search.model.SearchSuggestion
import app.indelible.search.model.rowKey
import app.indelible.search.repository.SearchRepository
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import kotlinx.serialization.json.Json
import kotlin.test.AfterTest
import kotlin.test.BeforeTest
import kotlin.test.Test
import kotlin.test.assertEquals

private const val WHEN = "2026-01-01T00:00:00Z"

private fun row(
    document: String,
    section: String?,
): String {
    val sectionJson = section?.let { """{"kind":"chapter","key":"$it","title":"$it"}""" } ?: "null"
    return """{"result_kind":"document","document_id":"$document","title":"Book","snippet":"…",""" +
        """"score":1.0,"content_type":"book","saved_at":"$WHEN","updated_at":"$WHEN","section":$sectionJson}"""
}

private class PagedSearch(
    private val pages: List<String>,
) : SearchRepository {
    private val json = Json { ignoreUnknownKeys = true }

    override suspend fun search(
        query: String,
        cursor: String?,
        limit: Int,
    ): Result<PaginatedSearchResults> {
        val index = cursor?.toInt() ?: 0
        val more = index + 1 < pages.size
        val next = if (more) "\"${index + 1}\"" else "null"
        val body = """{"query":"$query","results":[${pages[index]}],"has_more":$more,"next_cursor":$next}"""
        return Result.success(json.decodeFromString(PaginatedSearchResults.serializer(), body))
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
class SearchResultRowsTest {
    @BeforeTest
    fun setUp() {
        Dispatchers.setMain(UnconfinedTestDispatcher())
    }

    @AfterTest
    fun tearDown() {
        Dispatchers.resetMain()
    }

    @Test
    fun eachMatchingSectionOfABookIsItsOwnRow() =
        runTest {
            val viewModel = SearchViewModel(PagedSearch(listOf(row("doc_1", "c1") + "," + row("doc_1", "c2"))))

            viewModel.submitSearch("ships")
            advanceUntilIdle()

            val results = viewModel.state.value.results
            val keys = results.map { it.rowKey }
            assertEquals(2, keys.size)
            assertEquals(keys.size, keys.toSet().size)
        }

    @Test
    fun aRowTheServerRepeatsIsShownOnce() =
        runTest {
            val first = row("doc_1", null) + "," + row("doc_1", null)
            val pages = listOf(first, row("doc_1", null) + "," + row("doc_2", null))
            val viewModel = SearchViewModel(PagedSearch(pages))

            viewModel.submitSearch("ships")
            advanceUntilIdle()
            viewModel.loadNextPage()
            advanceUntilIdle()

            val results = viewModel.state.value.results
            assertEquals(listOf("doc_1", "doc_2"), results.map { it.documentId })
        }
}
