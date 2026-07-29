package app.indelible.profile.viewmodel

import app.indelible.profile.repository.AddLibraryRepository
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.CoroutineStart
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.async
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import kotlin.test.AfterTest
import kotlin.test.BeforeTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue

@OptIn(ExperimentalCoroutinesApi::class)
class AddLibraryViewModelTest {
    private val testDispatcher = UnconfinedTestDispatcher()

    @BeforeTest
    fun setUp() {
        Dispatchers.setMain(testDispatcher)
    }

    @AfterTest
    fun tearDown() {
        Dispatchers.resetMain()
    }

    @Test
    fun invalid_url_shows_inline_error_without_calling_repository() =
        runTest(testDispatcher) {
            listOf(
                "",
                "not a URL",
                "ftp://example.com/article",
                "https://",
                "https://exa mple.com",
                "https://.",
                "https://example.com/%zz",
                "https://[::::]/article",
                "https://[1::2::3]/article",
                "https://[12345::1]/article",
                "https://[::ffff:999.999.999.999]/article",
            ).forEach { invalidUrl ->
                val repository = FakeAddLibraryRepository()
                val viewModel = AddLibraryViewModel(repository)

                viewModel.save(invalidUrl)

                assertEquals(0, repository.calls, "Expected rejection for: $invalidUrl")
                assertFalse(viewModel.uiState.value.isSubmitting)
                assertEquals("Enter a valid http or https URL", viewModel.uiState.value.errorMessage)
            }
        }

    @Test
    fun submission_trims_url_and_ignores_a_duplicate_while_pending() =
        runTest(testDispatcher) {
            val repository = FakeAddLibraryRepository()
            val viewModel = AddLibraryViewModel(repository)

            viewModel.save("  https://example.com/article  ")
            viewModel.save("https://attacker.example/duplicate")

            assertTrue(viewModel.uiState.value.isSubmitting)
            assertEquals(1, repository.calls)
            assertEquals("https://example.com/article", repository.lastUrl)

            repository.complete(Result.success(Unit))
            advanceUntilIdle()
        }

    @Test
    fun valid_domain_and_ip_hosts_are_accepted() =
        runTest(testDispatcher) {
            listOf(
                "https://example.com/article?source=app#section",
                "http://localhost:8080/article",
                "http://127.0.0.1:8080/article",
                "http://[::1]:8080/article",
            ).forEach { validUrl ->
                val repository = FakeAddLibraryRepository()
                val viewModel = AddLibraryViewModel(repository)

                viewModel.save(validUrl)

                assertEquals(1, repository.calls, "Expected acceptance for: $validUrl")
                repository.complete(Result.success(Unit))
                advanceUntilIdle()
            }
        }

    @Test
    fun successful_submission_returns_to_idle_and_emits_saved() =
        runTest(testDispatcher) {
            val repository = FakeAddLibraryRepository()
            val viewModel = AddLibraryViewModel(repository)
            val effect = async(start = CoroutineStart.UNDISPATCHED) { viewModel.effects.first() }

            viewModel.save("https://example.com/article")
            repository.complete(Result.success(Unit))
            advanceUntilIdle()

            assertFalse(viewModel.uiState.value.isSubmitting)
            assertEquals(null, viewModel.uiState.value.errorMessage)
            assertEquals(AddLibraryEffect.Saved, effect.await())
        }

    @Test
    fun failed_submission_stays_open_with_a_retryable_inline_error() =
        runTest(testDispatcher) {
            val repository = FakeAddLibraryRepository()
            val viewModel = AddLibraryViewModel(repository)

            viewModel.save("https://example.com/article")
            repository.complete(Result.failure(IllegalStateException("Server unavailable")))
            advanceUntilIdle()

            assertFalse(viewModel.uiState.value.isSubmitting)
            assertEquals("Server unavailable", viewModel.uiState.value.errorMessage)
        }

    private class FakeAddLibraryRepository : AddLibraryRepository {
        private val pendingResult = CompletableDeferred<Result<Unit>>()
        var calls: Int = 0
            private set
        var lastUrl: String? = null
            private set

        override suspend fun save(url: String): Result<Unit> {
            calls += 1
            lastUrl = url
            return pendingResult.await()
        }

        fun complete(result: Result<Unit>) {
            pendingResult.complete(result)
        }
    }
}
