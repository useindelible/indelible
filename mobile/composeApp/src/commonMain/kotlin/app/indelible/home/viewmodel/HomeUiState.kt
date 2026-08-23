package app.indelible.home.viewmodel

import app.indelible.core.i18n.UiMessage
import app.indelible.home.model.HomeItem
import org.jetbrains.compose.resources.StringResource

enum class Greeting { MORNING, AFTERNOON, EVENING }

enum class StatIcon { READING_TIME, ITEMS_COMPLETED, STREAK }

data class StatTile(
    val labelRes: StringResource,
    val value: Long,
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
        val message: UiMessage,
    ) : HomeUiState()
}
