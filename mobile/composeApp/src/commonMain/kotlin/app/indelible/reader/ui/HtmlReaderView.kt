package app.indelible.reader.ui

import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import app.indelible.reader.model.HighlightData
import app.indelible.reader.model.ReaderPreferences
import kotlinx.datetime.Instant

@Composable
expect fun HtmlReaderView(
    htmlContent: String,
    highlights: List<HighlightData>,
    preferences: ReaderPreferences,
    isDarkMode: Boolean,
    scrollToPercent: Float?,
    anchorScroll: AnchorScrollRequest? = null,
    onScrollProgress: (percent: Float, scrollTopDp: Float) -> Unit,
    onTextSelected: (text: String, startOffset: Int, endOffset: Int, selectionRect: SelectionRect) -> Unit,
    onSelectionCleared: () -> Unit,
    onHighlightTapped: (highlightId: String, rect: SelectionRect) -> Unit,
    onContentLoaded: () -> Unit,
    onScrollRestored: () -> Unit,
    onAnchorScrolled: () -> Unit = {},
    onReaderTap: () -> Unit = {},
    articleTitle: String = "",
    articleAuthor: String? = null,
    articleDomain: String? = null,
    articlePublishedAt: Instant? = null,
    articleReadingTimeMinutes: Int? = null,
    summaryHtml: String? = null,
    summaryPoints: List<String> = emptyList(),
    onSummaryAction: (action: String) -> Unit = {},
    speakingSentenceIndex: Int = -1,
    immersive: Boolean = false,
    topContentPaddingPx: Int = 0,
    artwork: LoadedReaderArtwork? = null,
    modifier: Modifier = Modifier,
)

data class SelectionRect(
    val x: Float,
    val y: Float,
    val width: Float,
    val height: Float,
)

/**
 * One-shot anchor scroll: `nonce` distinguishes repeat taps on the same entry
 * so the consuming effect re-fires (mirrors the scrollToPercent consume
 * pattern).
 */
data class AnchorScrollRequest(
    val id: String,
    val fallbackIndex: Int,
    val nonce: Long,
)
