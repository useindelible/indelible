package app.indelible.reader.ui

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import app.indelible.reader.model.HighlightData
import app.indelible.reader.model.ReaderPreferences
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.reader_unavailable_desktop
import org.jetbrains.compose.resources.stringResource

@Composable
actual fun HtmlReaderView(
    htmlContent: String,
    highlights: List<HighlightData>,
    preferences: ReaderPreferences,
    isDarkMode: Boolean,
    scrollToPercent: Float?,
    anchorScroll: AnchorScrollRequest?,
    onScrollProgress: (Float, Float) -> Unit,
    onTextSelected: (text: String, startOffset: Int, endOffset: Int, selectionRect: SelectionRect) -> Unit,
    onSelectionCleared: () -> Unit,
    onHighlightTapped: (highlightId: String, rect: SelectionRect) -> Unit,
    onContentLoaded: () -> Unit,
    onScrollRestored: () -> Unit,
    onAnchorScrolled: () -> Unit,
    onReaderTap: () -> Unit,
    articleTitle: String,
    articleAuthor: String?,
    articleDomain: String?,
    localization: ReaderHtmlLocalization,
    summaryHtml: String?,
    summaryPoints: List<String>,
    onSummaryAction: (String) -> Unit,
    speakingSentenceIndex: Int,
    immersive: Boolean,
    topContentPaddingPx: Int,
    artwork: LoadedReaderArtwork?,
    modifier: Modifier,
) {
    Box(
        modifier = modifier.fillMaxSize(),
        contentAlignment = Alignment.Center,
    ) {
        Text(
            text = stringResource(Res.string.reader_unavailable_desktop),
            style = MaterialTheme.typography.bodyLarge,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
    }
}
