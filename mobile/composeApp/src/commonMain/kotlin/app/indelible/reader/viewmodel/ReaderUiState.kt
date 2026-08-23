package app.indelible.reader.viewmodel

import app.indelible.core.i18n.UiMessage
import app.indelible.reader.model.ArticleTocEntry
import app.indelible.reader.model.DocumentEntity
import app.indelible.reader.model.HighlightData
import app.indelible.reader.model.ReaderContentMode
import app.indelible.reader.model.ReaderDocument
import app.indelible.reader.model.ReaderPreferences

/**
 * Readable-content lifecycle for the HTML reader. The render is async (especially for
 * just-prepared feed documents), so the reader polls while [LOADING] and falls back to a
 * retryable [UNAVAILABLE] state once the poll budget is exhausted.
 */
enum class ReaderContentStatus { LOADING, READY, UNAVAILABLE }

enum class ReaderRetryStatus { IDLE, QUEUING, QUEUED, COOLDOWN }

/**
 * Contents-outline lifecycle. PENDING mirrors the backend's backfill contract
 * (the outline is being derived); NONE is terminal (too few headings) and
 * hides the Contents affordance entirely.
 */
enum class TocStatus { LOADING, PENDING, READY, NONE, UNAVAILABLE }

data class TocPanelState(
    val status: TocStatus = TocStatus.LOADING,
    val entries: List<ArticleTocEntry> = emptyList(),
    val truncated: Boolean = false,
    val activeIndex: Int = -1,
)

sealed class ReaderUiState {
    data object Loading : ReaderUiState()

    data class Success(
        val item: ReaderDocument,
        val htmlContent: String?,
        val contentMode: ReaderContentMode,
        val highlights: List<HighlightData>,
        val progress: Float,
        val preferences: ReaderPreferences,
        val itemNote: String? = null,
        val itemTags: List<String> = emptyList(),
        val entities: List<DocumentEntity> = emptyList(),
        val contentStatus: ReaderContentStatus = ReaderContentStatus.LOADING,
        val retryStatus: ReaderRetryStatus = ReaderRetryStatus.IDLE,
        val retryAfterSeconds: Long? = null,
        val toc: TocPanelState = TocPanelState(),
    ) : ReaderUiState()

    data class Error(
        val message: UiMessage,
    ) : ReaderUiState()
}

sealed class ReaderEffect {
    data object NavigateBack : ReaderEffect()

    data class ShowSnackbar(
        val message: UiMessage,
    ) : ReaderEffect()

    data class ScrollToPercent(
        val percent: Float,
    ) : ReaderEffect()

    data class ScrollToAnchor(
        val id: String,
        val fallbackIndex: Int,
    ) : ReaderEffect()
}
