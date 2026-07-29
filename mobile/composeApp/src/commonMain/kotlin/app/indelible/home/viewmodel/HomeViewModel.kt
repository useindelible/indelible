package app.indelible.home.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import app.indelible.home.model.ReadingStatsWidget
import app.indelible.home.repository.HomeRepository
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

class HomeViewModel(
    private val repository: HomeRepository,
    private val nowHour: () -> Int,
) : ViewModel() {
    private val _uiState = MutableStateFlow<HomeUiState>(HomeUiState.Loading)
    val uiState: StateFlow<HomeUiState> = _uiState.asStateFlow()

    fun load() {
        _uiState.value = HomeUiState.Loading
        viewModelScope.launch {
            repository
                .getDashboard()
                .onSuccess { dashboard ->
                    val continueItems = dashboard.continueReading?.items ?: emptyList()
                    _uiState.value =
                        HomeUiState.Ready(
                            greeting = greetingFor(nowHour()),
                            continueReading = continueItems.firstOrNull(),
                            stats = mapStats(dashboard.readingStats),
                            jumpBack = continueItems.drop(1),
                            recentlySaved = dashboard.recentlyAdded?.items ?: emptyList(),
                        )
                }.onFailure { error ->
                    _uiState.value = HomeUiState.Error(error.message ?: "Failed to load home")
                }
        }
    }

    private fun greetingFor(hour: Int): Greeting =
        when (hour) {
            in 0..11 -> Greeting.MORNING
            in 12..17 -> Greeting.AFTERNOON
            else -> Greeting.EVENING
        }

    private fun mapStats(stats: ReadingStatsWidget?): List<StatTile> {
        if (stats == null) return emptyList()
        return listOf(
            StatTile(
                label = "Read",
                value = stats.documentsRead.toString(),
                icon = StatIcon.READING_TIME,
            ),
            StatTile(
                label = "Finished",
                value = stats.itemsCompleted.toString(),
                icon = StatIcon.ITEMS_COMPLETED,
            ),
            StatTile(
                label = "Day streak",
                value = stats.streakDays.toString(),
                icon = StatIcon.STREAK,
            ),
        )
    }
}
