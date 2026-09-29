package app.indelible.reader.ui

import androidx.compose.animation.core.animateDpAsState
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.key
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import app.indelible.reader.model.HighlightData
import app.indelible.reader.ui.components.EpubChapterBar
import app.indelible.reader.ui.components.EpubStatusContent
import app.indelible.reader.viewmodel.EpubChapter
import app.indelible.reader.viewmodel.EpubReaderSession
import app.indelible.reader.viewmodel.EpubReaderState
import app.indelible.reader.viewmodel.EpubStatus
import app.indelible.reader.viewmodel.ReaderUiState
import app.indelible.reader.viewmodel.ReaderViewModel
import app.indelible.reader.viewmodel.highlightsFor
import app.indelible.ui.theme.IndelibleSpacing
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.reader_ask_follow_up
import indelible.composeapp.generated.resources.reader_epub_chapter_number
import indelible.composeapp.generated.resources.reader_summary_label
import org.jetbrains.compose.resources.stringResource

/** What the reader shows for a book: the chapter on screen, or why there is none. */
@Composable
internal fun ReaderEpubBody(
    state: ReaderUiState.Success,
    epub: EpubReaderState,
    session: EpubReaderSession,
    isDarkMode: Boolean,
    topChromeInsetPx: Int,
    onSelectedTextChanged: (SelectedTextInfo?) -> Unit,
    onTappedHighlightChanged: (HighlightData?) -> Unit,
    onReaderTap: () -> Unit,
    handleScrollProgress: (Float, Float) -> Unit,
    immersive: Boolean,
    viewModel: ReaderViewModel,
) {
    LaunchedEffect(epub.current, epub.status) {
        onSelectedTextChanged(null)
        onTappedHighlightChanged(null)
    }
    val chapter = epub.chapters.getOrNull(epub.current)
    val html = epub.html
    val barBottom by animateDpAsState(
        targetValue = if (immersive) IndelibleSpacing.step56 else IndelibleSpacing.step48 + IndelibleSpacing.step64,
        label = "epubChapterBarBottom",
    )
    Box(modifier = Modifier.fillMaxSize()) {
        Column(modifier = Modifier.fillMaxSize()) {
            EpubBodyContent(
                chapter = chapter,
                html = html,
                state = state,
                epub = epub,
                session = session,
                isDarkMode = isDarkMode,
                topChromeInsetPx = topChromeInsetPx,
                onSelectedTextChanged = onSelectedTextChanged,
                onTappedHighlightChanged = onTappedHighlightChanged,
                onReaderTap = onReaderTap,
                handleScrollProgress = handleScrollProgress,
                immersive = immersive,
                viewModel = viewModel,
            )
        }
        EpubChapterBar(
            epub,
            onPrevious = session::previous,
            onNext = session::next,
            modifier = Modifier.align(Alignment.BottomCenter).padding(bottom = barBottom),
        )
    }
}

@Composable
private fun ColumnScope.EpubBodyContent(
    chapter: EpubChapter?,
    html: String?,
    state: ReaderUiState.Success,
    epub: EpubReaderState,
    session: EpubReaderSession,
    isDarkMode: Boolean,
    topChromeInsetPx: Int,
    onSelectedTextChanged: (SelectedTextInfo?) -> Unit,
    onTappedHighlightChanged: (HighlightData?) -> Unit,
    onReaderTap: () -> Unit,
    handleScrollProgress: (Float, Float) -> Unit,
    immersive: Boolean,
    viewModel: ReaderViewModel,
) {
    when {
        chapter != null && html != null && epub.status == EpubStatus.READY ->
            key(chapter.spine) {
                EpubChapterView(
                    chapter = chapter,
                    html = html,
                    state = state,
                    epub = epub,
                    session = session,
                    isDarkMode = isDarkMode,
                    topChromeInsetPx = topChromeInsetPx,
                    onSelectedTextChanged = onSelectedTextChanged,
                    onTappedHighlightChanged = onTappedHighlightChanged,
                    onReaderTap = onReaderTap,
                    handleScrollProgress = handleScrollProgress,
                    immersive = immersive,
                    modifier = Modifier.weight(1f).fillMaxSize(),
                )
            }

        epub.status == EpubStatus.LOADING || epub.status == EpubStatus.READY ->
            Box(Modifier.fillMaxWidth().weight(1f), contentAlignment = Alignment.Center) {
                CircularProgressIndicator()
            }

        else ->
            EpubStatusContent(
                status = epub.status,
                onRetry = session::retry,
                onReprocess = viewModel::reprocessBook,
                modifier = Modifier.fillMaxWidth().weight(1f),
            )
    }
}

/** One chapter's view; every callback names that chapter, so a view being removed cannot speak for the next. */
@Composable
private fun EpubChapterView(
    chapter: EpubChapter,
    html: String,
    state: ReaderUiState.Success,
    epub: EpubReaderState,
    session: EpubReaderSession,
    isDarkMode: Boolean,
    topChromeInsetPx: Int,
    onSelectedTextChanged: (SelectedTextInfo?) -> Unit,
    onTappedHighlightChanged: (HighlightData?) -> Unit,
    onReaderTap: () -> Unit,
    handleScrollProgress: (Float, Float) -> Unit,
    immersive: Boolean,
    modifier: Modifier,
) {
    val spine = chapter.spine
    val highlights = epub.highlightsFor(state.highlights)
    HtmlReaderView(
        htmlContent = html,
        highlights = highlights,
        preferences = state.preferences,
        isDarkMode = isDarkMode,
        scrollToPercent = null,
        anchorScroll = epub.anchor,
        onScrollProgress = {
            handleScrollProgress(it.percent, it.scrollTopDp)
            session.onSample(spine, it)
        },
        onTextSelected = { text, start, end, rect ->
            onSelectedTextChanged(SelectedTextInfo(text = text, startOffset = start, endOffset = end, rect = rect))
        },
        onSelectionCleared = { onSelectedTextChanged(null) },
        onHighlightTapped = { id, _ -> onTappedHighlightChanged(highlights.find { it.id == id }) },
        onContentLoaded = {},
        onScrollRestored = {},
        onAnchorScrolled = {},
        onReaderTap = onReaderTap,
        articleTitle = chapter.title ?: stringResource(Res.string.reader_epub_chapter_number, epub.current + 1),
        articleAuthor = null,
        articleDomain = null,
        localization =
            ReaderHtmlLocalization(
                publishedDate = null,
                readingTime = null,
                summaryLabel = stringResource(Res.string.reader_summary_label),
                askFollowUpLabel = stringResource(Res.string.reader_ask_follow_up),
            ),
        summaryHtml = null,
        summaryPoints = emptyList(),
        onSummaryAction = {},
        speakingSentenceIndex = -1,
        immersive = immersive,
        topContentPaddingPx = topChromeInsetPx,
        artwork = null,
        scrollToCharOffset = epub.restore,
        captureRequest = epub.capture,
        onPositioned = { session.onPositioned(spine) },
        holdUntilPositioned = true,
        onBookLink = session::onBookLink,
        modifier = modifier,
    )
}
