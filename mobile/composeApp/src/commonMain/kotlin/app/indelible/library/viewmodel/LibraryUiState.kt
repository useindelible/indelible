package app.indelible.library.viewmodel

import app.indelible.core.i18n.UiMessage
import app.indelible.core.model.LibraryItem

enum class TriageFilter(
    val apiValue: String,
) {
    INBOX("inbox"),
    LATER("later"),
    ARCHIVE("archive"),
}

enum class ContentTypeFilter(
    val apiValue: String?,
) {
    ALL(null),
    ARTICLES("article"),
    BOOKS("book"),
    PDFS("pdf"),
    EMAILS("email"),
    TWEETS("tweet"),
    VIDEOS("video"),
    PODCASTS("podcast"),
    ;

    companion object {
        fun fromApiValue(apiValue: String?): ContentTypeFilter = entries.find { it.apiValue == apiValue } ?: ALL
    }
}

/**
 * What the library list is currently showing. [Triage] is the default browse mode
 * driven by the triage segmented control + content-type drawer; [Collection] and
 * [SmartList] pin the list to a specific saved view.
 *
 * The triage and content-type filters are deliberately kept as separate state
 * rather than wrapped inside [Triage]: they are orthogonal sub-state bound
 * directly by the segmented control and drawer, and folding them in here would
 * create two sources of truth for the same selection.
 */
sealed class LibraryScope {
    data object Triage : LibraryScope()

    data class Collection(
        val id: String,
        val name: String,
    ) : LibraryScope()

    data class SmartList(
        val id: String,
        val name: String,
    ) : LibraryScope()
}

sealed class LibraryUiState {
    data object Loading : LibraryUiState()

    data class Success(
        val items: List<LibraryItem>,
        val hasMore: Boolean,
        val isLoadingMore: Boolean = false,
        val isRefreshing: Boolean = false,
    ) : LibraryUiState()

    data class Error(
        val message: UiMessage,
    ) : LibraryUiState()
}

sealed class LibraryEffect {
    data class ShowSnackbar(
        val message: UiMessage,
    ) : LibraryEffect()
}
