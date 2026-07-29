package app.indelible.feed.viewmodel

import app.indelible.feed.model.FeedItemWithState

enum class FeedFilter(
    val apiValue: String,
) {
    UNSEEN("unseen"),
    SEEN("seen"),
}

sealed class FeedUiState {
    data object Loading : FeedUiState()

    data class Success(
        val items: List<FeedItemWithState>,
        val hasMore: Boolean,
        val isLoadingMore: Boolean = false,
        val isRefreshing: Boolean = false,
        val hasSubscriptions: Boolean = true,
        val savedItemIds: Set<String> = emptySet(),
        val sourceCount: Int? = null,
        val sourceCountExact: Boolean = true,
    ) : FeedUiState()

    data class Error(
        val message: String,
    ) : FeedUiState()
}

sealed class FeedEffect {
    data class ShowSnackbar(
        val message: String,
    ) : FeedEffect()

    data object NavigateToAddFeed : FeedEffect()

    data class OpenReader(
        val documentId: String,
    ) : FeedEffect()
}
