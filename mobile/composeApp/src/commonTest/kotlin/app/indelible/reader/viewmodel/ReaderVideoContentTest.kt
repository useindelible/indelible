package app.indelible.reader.viewmodel

import app.indelible.reader.model.DocumentEntity
import app.indelible.reader.viewmodel.FakeReaderRepository.Companion.fakeItemDetail
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
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
import kotlin.test.assertTrue

/**
 * Video documents are transcripts: the provider embed is hidden, transcript paragraph
 * rhythm is tightened, and Mila's extracted entities are loaded for the details sheet.
 * Articles must keep their existing rendering.
 */
@OptIn(ExperimentalCoroutinesApi::class)
class ReaderVideoContentTest {
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

    private fun entity(
        id: String,
        name: String,
        type: String,
    ) = DocumentEntity(
        createdAt = Instant.DISTANT_PAST,
        entityType = type,
        firstSeenAt = Instant.DISTANT_PAST,
        id = id,
        itemCount = 2,
        lastSeenAt = Instant.DISTANT_PAST,
        name = name,
        `object` = "entity",
        totalMentions = 2,
    )

    @Test
    fun videoTranscriptGetsTightenedParagraphRhythm() =
        runTest(testDispatcher) {
            repository.getItemResult = Result.success(fakeItemDetail(itemType = "video"))
            repository.fetchHtmlResult = Result.success("<p>Transcript line</p>")
            val viewModel = ReaderViewModel("doc_test1", repository)
            advanceUntilIdle()

            val html = assertIs<ReaderUiState.Success>(viewModel.uiState.value).htmlContent
            assertTrue(html!!.contains(".yt-embed{display:none!important}"), "provider embed should be hidden")
            assertTrue(
                html.contains("calc(var(--paragraph-spacing) * 0.65)"),
                "transcript paragraphs should tighten relative to the reader's spacing preference",
            )
            assertTrue(
                html.contains(".r-aura{display:none!important}"),
                "the poster is the cover art, so the masthead aura must not paint below it",
            )
        }

    @Test
    fun articleRenderingIsUnchanged() =
        runTest(testDispatcher) {
            repository.getItemResult = Result.success(fakeItemDetail(itemType = "article"))
            repository.fetchHtmlResult = Result.success("<p>Article content</p>")
            val viewModel = ReaderViewModel("doc_test1", repository)
            advanceUntilIdle()

            val html = assertIs<ReaderUiState.Success>(viewModel.uiState.value).htmlContent
            assertEquals("<p>Article content</p>", html)
            assertFalse(html!!.contains("paragraph-spacing) * 0.65"), "articles keep their own rhythm")
            assertFalse(html.contains(".r-aura{display:none"), "articles keep their masthead aura")
        }

    @Test
    fun entitiesAreLoadedIntoState() =
        runTest(testDispatcher) {
            repository.getItemResult = Result.success(fakeItemDetail(itemType = "video"))
            repository.listDocumentEntitiesResult =
                Result.success(
                    listOf(
                        entity("ent_1", "Sir Ken Robinson", "person"),
                        entity("ent_2", "TED", "organization"),
                    ),
                )
            val viewModel = ReaderViewModel("doc_test1", repository)
            advanceUntilIdle()

            val state = assertIs<ReaderUiState.Success>(viewModel.uiState.value)
            assertEquals(2, state.entities.size)
            assertEquals("Sir Ken Robinson", state.entities.first().name)
        }

    @Test
    fun entityFailureLeavesSectionEmptyWithoutError() =
        runTest(testDispatcher) {
            repository.getItemResult = Result.success(fakeItemDetail(itemType = "video"))
            repository.listDocumentEntitiesResult = Result.failure(RuntimeException("boom"))
            val viewModel = ReaderViewModel("doc_test1", repository)
            advanceUntilIdle()

            val state = assertIs<ReaderUiState.Success>(viewModel.uiState.value)
            assertTrue(state.entities.isEmpty(), "a failed entity fetch must not surface an error state")
        }
}
