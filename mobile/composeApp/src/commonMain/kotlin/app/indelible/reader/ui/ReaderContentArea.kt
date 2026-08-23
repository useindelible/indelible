package app.indelible.reader.ui

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.UriHandler
import app.indelible.core.i18n.LocaleFormatters
import app.indelible.core.i18n.LocalizedDateStyle
import app.indelible.reader.model.DataPanel
import app.indelible.reader.model.HighlightData
import app.indelible.reader.model.ReaderContentMode
import app.indelible.reader.playback.PlaybackState
import app.indelible.reader.ui.components.ReaderComingSoonContent
import app.indelible.reader.ui.components.ReaderComingSoonFormat
import app.indelible.reader.ui.components.ReaderPreparingContent
import app.indelible.reader.ui.components.YouTubeVideoHeader
import app.indelible.reader.viewmodel.ReaderContentStatus
import app.indelible.reader.viewmodel.ReaderRetryStatus
import app.indelible.reader.viewmodel.ReaderUiState
import app.indelible.reader.viewmodel.ReaderViewModel
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.reader_ask_follow_up
import indelible.composeapp.generated.resources.reader_minutes_short
import indelible.composeapp.generated.resources.reader_summary_label
import org.jetbrains.compose.resources.pluralStringResource
import org.jetbrains.compose.resources.stringResource

@Composable
internal fun ReaderContentArea(
    state: ReaderUiState.Success,
    playbackState: PlaybackState,
    isDarkMode: Boolean,
    scrollToPercent: Float?,
    anchorScroll: AnchorScrollRequest?,
    topChromeInsetPx: Int,
    handleScrollProgress: (Float, Float) -> Unit,
    onSelectedTextChanged: (SelectedTextInfo?) -> Unit,
    onScrollToPercentConsumed: () -> Unit,
    onAnchorScrollConsumed: () -> Unit,
    onTappedHighlightChanged: (HighlightData?) -> Unit,
    onReaderTap: () -> Unit,
    immersive: Boolean,
    uriHandler: UriHandler,
    viewModel: ReaderViewModel,
) {
    when (state.contentMode) {
        ReaderContentMode.HTML ->
            ReaderHtmlBody(
                state = state,
                playbackState = playbackState,
                isDarkMode = isDarkMode,
                scrollToPercent = scrollToPercent,
                anchorScroll = anchorScroll,
                topChromeInsetPx = topChromeInsetPx,
                handleScrollProgress = handleScrollProgress,
                onSelectedTextChanged = onSelectedTextChanged,
                onScrollToPercentConsumed = onScrollToPercentConsumed,
                onAnchorScrollConsumed = onAnchorScrollConsumed,
                onTappedHighlightChanged = onTappedHighlightChanged,
                onReaderTap = onReaderTap,
                immersive = immersive,
                uriHandler = uriHandler,
                viewModel = viewModel,
            )

        ReaderContentMode.PDF_COMING_SOON ->
            ReaderComingSoonContent(
                format = ReaderComingSoonFormat.PDF,
                modifier = Modifier.fillMaxSize(),
            )

        ReaderContentMode.EPUB_COMING_SOON ->
            ReaderComingSoonContent(
                format = ReaderComingSoonFormat.EPUB,
                modifier = Modifier.fillMaxSize(),
            )
    }
}

@Composable
private fun ReaderHtmlBody(
    state: ReaderUiState.Success,
    playbackState: PlaybackState,
    isDarkMode: Boolean,
    scrollToPercent: Float?,
    anchorScroll: AnchorScrollRequest?,
    topChromeInsetPx: Int,
    handleScrollProgress: (Float, Float) -> Unit,
    onSelectedTextChanged: (SelectedTextInfo?) -> Unit,
    onScrollToPercentConsumed: () -> Unit,
    onAnchorScrollConsumed: () -> Unit,
    onTappedHighlightChanged: (HighlightData?) -> Unit,
    onReaderTap: () -> Unit,
    immersive: Boolean,
    uriHandler: UriHandler,
    viewModel: ReaderViewModel,
) {
    Column(modifier = Modifier.fillMaxSize()) {
        val isVideo = state.item.itemType == "video"
        if (isVideo) {
            // The poster is the video's cover art, so it takes the top of the frame the
            // way an article's aura does — full-bleed under the chrome, which carries its
            // own scrim to stay legible over the image.
            YouTubeVideoHeader(
                thumbnailUrl = state.item.thumbnailUrl ?: state.item.leadImageUrl,
                durationSeconds = state.item.videoDurationSeconds,
                onPlayTapped = {
                    val url = state.item.canonicalUrl ?: state.item.url
                    if (url != null) uriHandler.openUri(url)
                },
            )
        }
        // A video carries a native poster instead of a drawing. For everything else the
        // web view waits for the artwork: rendering without it and swapping later would
        // rebuild the document and drop the reader's scroll position.
        val artwork = if (isVideo) null else rememberReaderArtwork(state.item.documentId)
        val html = state.htmlContent
        if (html != null && (isVideo || artwork != null)) {
            ReaderArticleWebView(
                html = html,
                state = state,
                playbackState = playbackState,
                isDarkMode = isDarkMode,
                scrollToPercent = scrollToPercent,
                anchorScroll = anchorScroll,
                topContentPaddingPx = if (isVideo) 0 else topChromeInsetPx,
                isVideo = isVideo,
                artwork = artwork,
                handleScrollProgress = handleScrollProgress,
                onSelectedTextChanged = onSelectedTextChanged,
                onScrollToPercentConsumed = onScrollToPercentConsumed,
                onAnchorScrollConsumed = onAnchorScrollConsumed,
                onTappedHighlightChanged = onTappedHighlightChanged,
                onReaderTap = onReaderTap,
                immersive = immersive,
                viewModel = viewModel,
                modifier = Modifier.weight(1f).fillMaxSize(),
            )
        } else {
            ReaderContentPlaceholder(state = state, viewModel = viewModel)
        }
    }
}

@Composable
private fun ColumnScope.ReaderContentPlaceholder(
    state: ReaderUiState.Success,
    viewModel: ReaderViewModel,
) {
    val unavailable =
        state.contentStatus == ReaderContentStatus.UNAVAILABLE ||
            state.retryStatus != ReaderRetryStatus.IDLE
    if (unavailable) {
        ReaderPreparingContent(
            onRetry = { viewModel.retryLoadContent() },
            retryStatus = state.retryStatus,
            retryAfterSeconds = state.retryAfterSeconds,
            modifier = Modifier.fillMaxWidth().weight(1f),
        )
    } else {
        Box(
            modifier = Modifier.fillMaxWidth().weight(1f),
            contentAlignment = Alignment.Center,
        ) {
            CircularProgressIndicator()
        }
    }
}

@Composable
private fun ReaderArticleWebView(
    html: String,
    state: ReaderUiState.Success,
    playbackState: PlaybackState,
    isDarkMode: Boolean,
    scrollToPercent: Float?,
    anchorScroll: AnchorScrollRequest?,
    topContentPaddingPx: Int,
    isVideo: Boolean,
    artwork: LoadedReaderArtwork?,
    handleScrollProgress: (Float, Float) -> Unit,
    onSelectedTextChanged: (SelectedTextInfo?) -> Unit,
    onScrollToPercentConsumed: () -> Unit,
    onAnchorScrollConsumed: () -> Unit,
    onTappedHighlightChanged: (HighlightData?) -> Unit,
    onReaderTap: () -> Unit,
    immersive: Boolean,
    viewModel: ReaderViewModel,
    modifier: Modifier = Modifier,
) {
    val readingTimeMinutes = if (isVideo) null else state.item.readingTimeMinutes
    val localization =
        ReaderHtmlLocalization(
            publishedDate =
                if (isVideo) {
                    null
                } else {
                    state.item.publishedAt?.let { LocaleFormatters.date(it, LocalizedDateStyle.MEDIUM) }
                },
            readingTime =
                readingTimeMinutes?.let { minutes ->
                    pluralStringResource(Res.plurals.reader_minutes_short, minutes, minutes)
                },
            summaryLabel = stringResource(Res.string.reader_summary_label),
            askFollowUpLabel = stringResource(Res.string.reader_ask_follow_up),
        )
    HtmlReaderView(
        htmlContent = html,
        highlights = state.highlights,
        preferences = state.preferences,
        isDarkMode = isDarkMode,
        scrollToPercent = scrollToPercent,
        anchorScroll = anchorScroll,
        onScrollProgress = handleScrollProgress,
        onTextSelected = { text, start, end, rect ->
            onSelectedTextChanged(
                SelectedTextInfo(text = text, startOffset = start, endOffset = end, rect = rect),
            )
        },
        onSelectionCleared = { onSelectedTextChanged(null) },
        onHighlightTapped = { highlightId, _ ->
            onTappedHighlightChanged(state.highlights.find { it.id == highlightId })
        },
        onContentLoaded = { viewModel.onContentLoaded() },
        onScrollRestored = { onScrollToPercentConsumed() },
        onAnchorScrolled = { onAnchorScrollConsumed() },
        onReaderTap = onReaderTap,
        // A video body carries its own channel header (channel, views, duration),
        // so the masthead contributes only the title and leaves the rest to it.
        articleTitle = state.item.title,
        articleAuthor = if (isVideo) null else state.item.author,
        articleDomain = if (isVideo) null else state.item.domain,
        localization = localization,
        summaryHtml = state.item.summary,
        // summaryPoints left empty: the library reader exposes only
        // a single summary string, no structured key-points array.
        // The template renders the bullet list once the backend adds one.
        onSummaryAction = { action ->
            if (action == "ask") viewModel.openPanel(DataPanel.MILA)
        },
        speakingSentenceIndex =
            if (playbackState.isPlaying) playbackState.currentSentenceIndex else -1,
        immersive = immersive,
        topContentPaddingPx = topContentPaddingPx,
        artwork = artwork,
        modifier = modifier,
    )
}
