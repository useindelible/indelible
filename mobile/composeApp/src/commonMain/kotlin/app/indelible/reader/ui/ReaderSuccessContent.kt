package app.indelible.reader.ui

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.core.animateDpAsState
import androidx.compose.animation.expandVertically
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.shrinkVertically
import androidx.compose.animation.slideInVertically
import androidx.compose.animation.slideOutVertically
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.asPaddingValues
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.statusBars
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.SnackbarHostState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalClipboardManager
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.text.AnnotatedString
import app.indelible.mila.viewmodel.MilaChatViewModel
import app.indelible.reader.model.DataPanel
import app.indelible.reader.model.HighlightColor
import app.indelible.reader.model.HighlightData
import app.indelible.reader.model.ReaderBackground
import app.indelible.reader.model.TagData
import app.indelible.reader.playback.PlaybackState
import app.indelible.reader.ui.components.showContentsPill
import app.indelible.reader.ui.components.HighlightSheet
import app.indelible.reader.ui.components.HighlightTagSheet
import app.indelible.reader.ui.components.MilaReaderDrawer
import app.indelible.reader.ui.components.ReaderBottomChrome
import app.indelible.reader.ui.components.ReaderDock
import app.indelible.reader.ui.components.ReaderDockPanels
import app.indelible.reader.ui.components.ReaderFloatingControls
import app.indelible.reader.ui.components.TtsMiniBar
import app.indelible.reader.viewmodel.ReaderUiState
import app.indelible.reader.viewmodel.ReaderViewModel
import app.indelible.ui.theme.AppTheme
import app.indelible.ui.theme.IndelibleSpacing
import app.indelible.ui.theme.IndelibleTheme
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch

internal data class SelectedTextInfo(
    val text: String,
    val startOffset: Int,
    val endOffset: Int,
    val rect: SelectionRect,
)

@Composable
internal fun ReaderSuccessContent(
    state: ReaderUiState.Success,
    activePanel: DataPanel,
    defaultHighlightColor: HighlightColor,
    playbackState: PlaybackState,
    availableTags: List<TagData>,
    immersive: Boolean,
    selectedText: SelectedTextInfo?,
    tappedHighlight: HighlightData?,
    tagSheetHighlightId: String?,
    milaStarted: Boolean,
    scrollToPercent: Float?,
    anchorScroll: AnchorScrollRequest?,
    isDarkMode: Boolean,
    snackbarHostState: SnackbarHostState,
    coroutineScope: CoroutineScope,
    viewModel: ReaderViewModel,
    milaViewModelProvider: (title: String) -> MilaChatViewModel,
    onNavigateToAiSettings: () -> Unit,
    onNavigateToItem: (String) -> Unit,
    onSelectedTextChanged: (SelectedTextInfo?) -> Unit,
    onTappedHighlightChanged: (HighlightData?) -> Unit,
    onTagSheetHighlightIdChanged: (String?) -> Unit,
    onAvailableTagsChanged: (List<TagData>) -> Unit,
    onScrollToPercentConsumed: () -> Unit,
    onAnchorScrollConsumed: () -> Unit,
    handleScrollProgress: (Float, Float) -> Unit,
    onReaderTap: () -> Unit,
) {
    val uriHandler = LocalUriHandler.current
    val clipboardManager = LocalClipboardManager.current

    val readerIsDark =
        state.preferences.background == ReaderBackground.SLATE ||
            state.preferences.background == ReaderBackground.BLACK

    // The article runs to the status bar so the drawing paints from the very top;
    // only the prose is pushed clear of the floating controls that sit over it.
    // step48 is the pill band: step8 of top padding plus a step40 pill.
    val topChromeInsetPx =
        (
            WindowInsets.statusBars.asPaddingValues().calculateTopPadding() +
                IndelibleSpacing.step48
        ).value.toInt()

    // Reader chrome contrast follows the chosen reader canvas, not the
    // system theme: SLATE/BLACK are dark surfaces, PAPER/SEPIA are light.
    AppTheme(darkTheme = readerIsDark) {
        Box(
            modifier =
                Modifier
                    .fillMaxSize()
                    .background(IndelibleTheme.colors.readerBg),
        ) {
            Column(modifier = Modifier.fillMaxSize()) {
                Box(modifier = Modifier.weight(1f)) {
                    ReaderContentArea(
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

                    val sel = selectedText
                    if (sel != null) {
                        HighlightToolbarOverlay(
                            sel = sel,
                            defaultHighlightColor = defaultHighlightColor,
                            snackbarHostState = snackbarHostState,
                            coroutineScope = coroutineScope,
                            clipboardManager = clipboardManager,
                            viewModel = viewModel,
                            onSelectedTextChanged = onSelectedTextChanged,
                            onAvailableTagsChanged = onAvailableTagsChanged,
                            onTagSheetHighlightIdChanged = onTagSheetHighlightIdChanged,
                        )
                    }
                }
            }

            // Deliberately outside the immersive blocks: when the furniture leaves,
            // what you are reading and how far in you are both stay.
            ReaderBottomChrome(
                title = state.item.title,
                progress = state.progress,
                modifier = Modifier.align(Alignment.BottomCenter),
            )

            AnimatedVisibility(
                visible = !immersive,
                enter = fadeIn() + expandVertically(),
                exit = fadeOut() + shrinkVertically(),
                modifier = Modifier.align(Alignment.TopCenter),
            ) {
                ReaderFloatingControls(
                    onBack = { viewModel.navigateBack() },
                    onContents = { viewModel.openPanel(DataPanel.CONTENTS) },
                    showContents = showContentsPill(state.toc.status),
                    onMore = { viewModel.openPanel(DataPanel.INFO) },
                    canSave = !state.item.saved && state.item.url != null,
                    onSave = { viewModel.saveToLibrary() },
                )
            }

            AnimatedVisibility(
                visible = !immersive,
                enter = fadeIn() + slideInVertically { it },
                exit = fadeOut() + slideOutVertically { it },
                modifier =
                    Modifier
                        .align(Alignment.BottomCenter)
                        // Clears the standing-title shelf below it, so the dock reads as
                        // sitting on the shelf rather than overlapping the title. This
                        // already keeps the interactive dock above the system gesture area.
                        .padding(bottom = IndelibleSpacing.step48),
            ) {
                ReaderDock(
                    activePanel = activePanel,
                    onPanelSelected = { viewModel.openPanel(it) },
                )
            }

            val showMiniBar =
                (playbackState.isPlaying || playbackState.positionMs > 0L) &&
                    activePanel != DataPanel.LISTEN
            val miniBarBottom by animateDpAsState(
                targetValue = if (immersive) IndelibleSpacing.step16 else IndelibleSpacing.step80,
                label = "ttsMiniBarBottom",
            )
            AnimatedVisibility(
                visible = showMiniBar,
                enter = fadeIn() + slideInVertically { it },
                exit = fadeOut() + slideOutVertically { it },
                modifier =
                    Modifier
                        .align(Alignment.BottomCenter)
                        .navigationBarsPadding()
                        .padding(horizontal = IndelibleSpacing.step12)
                        .padding(bottom = miniBarBottom),
            ) {
                TtsMiniBar(
                    title = state.item.title,
                    voiceName =
                        viewModel.voices
                            .firstOrNull { it.id == playbackState.voiceId }
                            ?.name ?: "Voice",
                    isPlaying = playbackState.isPlaying,
                    progressFraction = playbackState.progressFraction,
                    onTogglePlay = { viewModel.togglePlayback() },
                    onExpand = { viewModel.openPanel(DataPanel.LISTEN) },
                    onClose = { viewModel.stopPlayback() },
                )
            }
        }

        ReaderDockPanels(
            activePanel = activePanel,
            state = state,
            defaultHighlightColor = defaultHighlightColor,
            availableTags = availableTags,
            playbackState = playbackState,
            voices = viewModel.voices,
            onDismiss = { viewModel.closePanel() },
            onPreferencesChanged = { viewModel.updatePreferences(it) },
            onDefaultHighlightColorSelected = { viewModel.setDefaultHighlightColor(it) },
            onSaveNote = { viewModel.saveItemNote(it) },
            onTagsChanged = { viewModel.setItemTags(it) },
            onEditNote = { viewModel.openPanel(DataPanel.NOTE) },
            onMove = { viewModel.moveToTriage(it) },
            onSaveToLibrary = { viewModel.saveToLibrary() },
            onTogglePlay = { viewModel.togglePlayback() },
            onSeek = { viewModel.seekPlayback(it) },
            onSkip = { viewModel.skipPlayback(it) },
            onSetSpeed = { viewModel.setPlaybackSpeed(it) },
            onSelectVoice = { viewModel.selectVoice(it) },
            onSetSleepTimer = { viewModel.setSleepTimer(it) },
            onTocEntryTapped = { viewModel.onTocEntryTapped(it) },
            onShare = {
                val url = state.item.canonicalUrl ?: state.item.url
                if (url != null) {
                    clipboardManager.setText(AnnotatedString(url))
                    coroutineScope.launch { snackbarHostState.showSnackbar("Link copied to clipboard") }
                } else {
                    coroutineScope.launch { snackbarHostState.showSnackbar("No link to share") }
                }
            },
        )

        val highlight = tappedHighlight
        if (highlight != null) {
            HighlightSheet(
                highlight = highlight,
                onColorChanged = { color ->
                    viewModel.updateHighlightColor(highlight.id, color)
                    onTappedHighlightChanged(null)
                },
                onEditNote = {
                    onTappedHighlightChanged(null)
                },
                onTagsSelected = {
                    val hlId = highlight.id
                    onTappedHighlightChanged(null)
                    viewModel.loadTagsForPicker { tags ->
                        onAvailableTagsChanged(tags)
                        onTagSheetHighlightIdChanged(hlId)
                    }
                },
                onDelete = {
                    viewModel.deleteHighlight(highlight.id)
                    onTappedHighlightChanged(null)
                },
                onCopy = {
                    clipboardManager.setText(AnnotatedString(highlight.textContent))
                    onTappedHighlightChanged(null)
                    coroutineScope.launch {
                        snackbarHostState.showSnackbar("Copied to clipboard")
                    }
                },
                onDismiss = { onTappedHighlightChanged(null) },
            )
        }

        val hlId = tagSheetHighlightId
        val tagHighlight = if (hlId != null) state.highlights.find { it.id == hlId } else null
        if (tagHighlight != null) {
            HighlightTagSheet(
                appliedTags = tagHighlight.tags,
                availableTags = availableTags,
                onTagsChanged = { tags ->
                    viewModel.setHighlightTags(tagHighlight.id, tags)
                },
                onDismiss = { onTagSheetHighlightIdChanged(null) },
            )
        }

        if (milaStarted) {
            val milaViewModel = remember { milaViewModelProvider(state.item.title) }
            MilaReaderDrawer(
                visible = activePanel == DataPanel.MILA,
                title = state.item.title,
                viewModel = milaViewModel,
                onDismiss = { viewModel.closePanel() },
                onNavigateToAiSettings = onNavigateToAiSettings,
                onNavigateToItem = onNavigateToItem,
            )
        }
    }
}
