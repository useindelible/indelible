package app.indelible.mila.viewmodel

import app.indelible.core.i18n.UiMessage
import app.indelible.mila.data.ChatMessage

data class MilaChatUiState(
    val configLoading: Boolean = true,
    val milaEnabled: Boolean = false,
    val sessionLoading: Boolean = false,
    val messages: List<ChatMessage> = emptyList(),
    val isStreaming: Boolean = false,
    val inputText: String = "",
    val error: UiMessage? = null,
)

sealed class MilaChatEffect {
    data object NavigateToAiSettings : MilaChatEffect()

    data class NavigateToItem(
        val itemId: String,
    ) : MilaChatEffect()
}
