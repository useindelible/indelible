package app.indelible.home.viewmodel

import app.indelible.core.i18n.UiMessage
import app.indelible.home.viewmodel.FakeHomeRepository.Companion.sampleDashboard
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.home_error_load
import indelible.composeapp.generated.resources.home_stat_read
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
import kotlin.test.assertTrue

@OptIn(ExperimentalCoroutinesApi::class)
class HomeViewModelTest {
    private val testDispatcher = UnconfinedTestDispatcher()
    private lateinit var repository: FakeHomeRepository

    @BeforeTest
    fun setUp() {
        Dispatchers.setMain(testDispatcher)
        repository = FakeHomeRepository()
    }

    @AfterTest
    fun tearDown() {
        Dispatchers.resetMain()
    }

    @Test
    fun maps_dashboard_to_ready_state() =
        runTest(testDispatcher) {
            repository.dashboardResult = Result.success(sampleDashboard())

            val viewModel = HomeViewModel(repository, nowHour = { 9 })
            viewModel.load()
            advanceUntilIdle()

            val state = assertIs<HomeUiState.Ready>(viewModel.uiState.value)
            assertEquals(Greeting.MORNING, state.greeting)
            assertNotNull(state.continueReading)
            assertEquals(3, state.stats.size)
            assertEquals(1, state.jumpBack.size)
            assertEquals(1, state.recentlySaved.size)
        }

    @Test
    fun greeting_tracks_the_injected_clock() =
        runTest(testDispatcher) {
            repository.dashboardResult = Result.success(sampleDashboard())

            val afternoon = HomeViewModel(repository, nowHour = { 14 })
            afternoon.load()
            advanceUntilIdle()
            assertEquals(
                Greeting.AFTERNOON,
                assertIs<HomeUiState.Ready>(afternoon.uiState.value).greeting,
            )

            val evening = HomeViewModel(repository, nowHour = { 20 })
            evening.load()
            advanceUntilIdle()
            assertEquals(
                Greeting.EVENING,
                assertIs<HomeUiState.Ready>(evening.uiState.value).greeting,
            )
        }

    @Test
    fun read_stat_shows_documents_read_count() =
        runTest(testDispatcher) {
            repository.dashboardResult = Result.success(sampleDashboard())

            val viewModel = HomeViewModel(repository, nowHour = { 9 })
            viewModel.load()
            advanceUntilIdle()

            val state = assertIs<HomeUiState.Ready>(viewModel.uiState.value)
            val read = state.stats.first { it.icon == StatIcon.READING_TIME }
            assertEquals(7L, read.value)
            assertEquals(Res.string.home_stat_read, read.labelRes)
        }

    @Test
    fun missing_widgets_yield_empty_sections_without_error() =
        runTest(testDispatcher) {
            repository.dashboardResult = Result.success(FakeHomeRepository.emptyDashboard())

            val viewModel = HomeViewModel(repository, nowHour = { 9 })
            viewModel.load()
            advanceUntilIdle()

            val state = assertIs<HomeUiState.Ready>(viewModel.uiState.value)
            assertNull(state.continueReading)
            assertTrue(state.stats.isEmpty())
            assertTrue(state.jumpBack.isEmpty())
            assertTrue(state.recentlySaved.isEmpty())
        }

    @Test
    fun failure_yields_error_state() =
        runTest(testDispatcher) {
            repository.dashboardResult = Result.failure(RuntimeException("offline"))

            val viewModel = HomeViewModel(repository, nowHour = { 9 })
            viewModel.load()
            advanceUntilIdle()

            val state = assertIs<HomeUiState.Error>(viewModel.uiState.value)
            assertEquals(UiMessage(Res.string.home_error_load), state.message)
        }
}
