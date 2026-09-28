package app.indelible.reader.ui

import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.ui.Modifier
import app.indelible.reader.model.HighlightData
import app.indelible.reader.model.ReaderPreferences

@Composable
expect fun HtmlReaderView(
    htmlContent: String,
    highlights: List<HighlightData>,
    preferences: ReaderPreferences,
    isDarkMode: Boolean,
    scrollToPercent: Float?,
    anchorScroll: AnchorScrollRequest? = null,
    onScrollProgress: (ScrollSample) -> Unit,
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
    localization: ReaderHtmlLocalization,
    summaryHtml: String? = null,
    summaryPoints: List<String> = emptyList(),
    onSummaryAction: (action: String) -> Unit = {},
    speakingSentenceIndex: Int = -1,
    immersive: Boolean = false,
    topContentPaddingPx: Int = 0,
    artwork: LoadedReaderArtwork? = null,
    scrollToCharOffset: CharOffsetRequest? = null,
    captureRequest: Long? = null,
    onPositioned: () -> Unit = {},
    holdUntilPositioned: Boolean = false,
    onBookLink: (href: String) -> Unit = {},
    modifier: Modifier = Modifier,
)

/** Runs a book's restore and capture requests once the page has loaded; [evaluate] runs a script in it. */
@Composable
internal fun BookBridgeEffects(
    pageLoaded: Boolean,
    scrollToCharOffset: CharOffsetRequest?,
    captureRequest: Long?,
    evaluate: (String) -> Unit,
) {
    LaunchedEffect(scrollToCharOffset, pageLoaded) {
        val restore = scrollToCharOffset ?: return@LaunchedEffect
        if (pageLoaded) evaluate("window.scrollToCharOffset(${restore.offset});")
    }
    LaunchedEffect(captureRequest) {
        if (captureRequest != null && pageLoaded) evaluate("window.captureSample();")
    }
}

/** What the page calls in native code; each platform bridge forwards to these. */
internal data class ReaderBridgeCallbacks(
    val onScroll: (ScrollSample) -> Unit,
    val onTextSelected: (String, Int, Int, SelectionRect) -> Unit,
    val onSelectionCleared: () -> Unit,
    val onHighlightTapped: (String, SelectionRect) -> Unit,
    val onSummaryAction: (String) -> Unit,
    val onReaderTap: () -> Unit,
    val onPositioned: () -> Unit,
    val onBookLink: (String) -> Unit,
)

/** One scroll report; [firstVisibleOffset] is -1 on a frame and the first visible character on a complete sample. */
data class ScrollSample(
    val percent: Float,
    val scrollTopDp: Float,
    val firstVisibleOffset: Int,
    val atEnd: Boolean,
)

/** One-shot restore to a character of the chapter text; [nonce] re-fires a repeat of the same offset. */
data class CharOffsetRequest(
    val offset: Int,
    val nonce: Long,
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
