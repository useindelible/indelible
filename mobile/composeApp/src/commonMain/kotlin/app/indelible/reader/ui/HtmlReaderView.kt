package app.indelible.reader.ui

import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberUpdatedState
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
    assets: ReaderAssets = ReaderAssets.NONE,
    scrollToCharOffset: CharOffsetRequest? = null,
    captureRequest: Long? = null,
    onPositioned: () -> Unit = {},
    holdUntilPositioned: Boolean = false,
    onBookLink: (href: String) -> Unit = {},
    modifier: Modifier = Modifier,
)

/** The page's inputs that change while it is shown; each one becomes a script call through [evaluate]. */
internal data class ReaderBridgeInputs(
    val pageLoaded: Boolean,
    val scrollToPercent: Float?,
    val anchorScroll: AnchorScrollRequest?,
    val preferences: ReaderPreferences,
    val highlights: List<HighlightData>,
    val speakingSentenceIndex: Int,
    val immersive: Boolean,
    val scrollToCharOffset: CharOffsetRequest?,
    val captureRequest: Long?,
)

/** Forwards every change in [inputs] to the page; one-shot requests report back through the two callbacks. */
@Composable
internal fun ReaderBridgeEffects(
    inputs: ReaderBridgeInputs,
    onScrollRestored: () -> Unit,
    onAnchorScrolled: () -> Unit,
    evaluate: (String) -> Unit,
) {
    val pageLoaded = inputs.pageLoaded
    LaunchedEffect(inputs.scrollToPercent) {
        val percent = inputs.scrollToPercent ?: return@LaunchedEffect
        evaluate("window.scrollToPercent($percent);")
        onScrollRestored()
    }
    LaunchedEffect(inputs.anchorScroll, pageLoaded) {
        val anchor = inputs.anchorScroll ?: return@LaunchedEffect
        if (!pageLoaded) return@LaunchedEffect
        evaluate("window.scrollToAnchor('${escapeSingleQuoted(anchor.id)}', ${anchor.fallbackIndex});")
        onAnchorScrolled()
    }
    LaunchedEffect(inputs.preferences) {
        val css = escapeSingleQuoted(ReaderHtmlTemplate.buildTypographyCss(inputs.preferences)).replace("\n", "\\n")
        evaluate("window.updateTypography('$css', '${ReaderHtmlTemplate.colorSchemeFor(inputs.preferences)}');")
    }
    LaunchedEffect(inputs.highlights) {
        evaluate("window.applyHighlights(${ReaderHtmlTemplate.buildHighlightJson(inputs.highlights)});")
    }
    LaunchedEffect(inputs.speakingSentenceIndex) {
        evaluate("window.setSpeaking(${inputs.speakingSentenceIndex});")
    }
    LaunchedEffect(inputs.immersive) {
        evaluate("window.setReaderImmersive(${inputs.immersive});")
    }
    LaunchedEffect(inputs.scrollToCharOffset, pageLoaded) {
        val restore = inputs.scrollToCharOffset ?: return@LaunchedEffect
        if (pageLoaded) evaluate("window.scrollToCharOffset(${restore.offset});")
    }
    LaunchedEffect(inputs.captureRequest) {
        if (inputs.captureRequest != null && pageLoaded) evaluate("window.captureSample();")
    }
}

private fun escapeSingleQuoted(value: String): String = value.replace("\\", "\\\\").replace("'", "\\'")

/** The latest lambdas, so a bridge created once still calls the current ones. */
@Composable
internal fun rememberReaderBridgeCallbacks(
    onScroll: (ScrollSample) -> Unit,
    onTextSelected: (String, Int, Int, SelectionRect) -> Unit,
    onSelectionCleared: () -> Unit,
    onHighlightTapped: (String, SelectionRect) -> Unit,
    onSummaryAction: (String) -> Unit,
    onReaderTap: () -> Unit,
    onPositioned: () -> Unit,
    onBookLink: (String) -> Unit,
): ReaderBridgeCallbacks {
    val scroll = rememberUpdatedState(onScroll)
    val selected = rememberUpdatedState(onTextSelected)
    val cleared = rememberUpdatedState(onSelectionCleared)
    val tapped = rememberUpdatedState(onHighlightTapped)
    val summary = rememberUpdatedState(onSummaryAction)
    val readerTap = rememberUpdatedState(onReaderTap)
    val positioned = rememberUpdatedState(onPositioned)
    val bookLink = rememberUpdatedState(onBookLink)
    return remember {
        ReaderBridgeCallbacks(
            onScroll = { scroll.value(it) },
            onTextSelected = { t, s, e, r -> selected.value(t, s, e, r) },
            onSelectionCleared = { cleared.value() },
            onHighlightTapped = { id, r -> tapped.value(id, r) },
            onSummaryAction = { summary.value(it) },
            onReaderTap = { readerTap.value() },
            onPositioned = { positioned.value() },
            onBookLink = { bookLink.value(it) },
        )
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
