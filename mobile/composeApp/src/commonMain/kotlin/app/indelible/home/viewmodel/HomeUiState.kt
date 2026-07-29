package app.indelible.home.viewmodel

import app.indelible.home.model.HomeItem

enum class Greeting { MORNING, AFTERNOON, EVENING }

enum class StatIcon { READING_TIME, ITEMS_COMPLETED, STREAK }

data class StatTile(
    val label: String,
    val value: String,
    val icon: StatIcon,
)

sealed class HomeUiState {
    data object Loading : HomeUiState()

    data class Ready(
        val greeting: Greeting,
        val continueReading: HomeItem?,
        val stats: List<StatTile>,
        val jumpBack: List<HomeItem>,
        val recentlySaved: List<HomeItem>,
    ) : HomeUiState()

    data class Error(
        val message: String,
    ) : HomeUiState()
}
