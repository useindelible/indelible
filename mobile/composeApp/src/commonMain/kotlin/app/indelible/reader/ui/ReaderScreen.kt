package app.indelible.reader.ui

import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import app.indelible.mila.viewmodel.MilaChatViewModel
import app.indelible.reader.model.DataPanel
import app.indelible.reader.model.HighlightData
import app.indelible.reader.model.TagData
import app.indelible.reader.viewmodel.ReaderEffect
import app.indelible.reader.viewmodel.ReaderUiState
import app.indelible.reader.viewmodel.ReaderViewModel
import app.indelible.ui.platform.ImmersiveSystemBars

@Composable
fun ReaderScreen(
    viewModel: ReaderViewModel,
    onNavigateBack: () -> Unit,
    milaViewModelProvider: (title: String) -> MilaChatViewModel,
    onNavigateToAiSettings: () -> Unit,
    onNavigateToItem: (String) -> Unit,
    modifier: Modifier = Modifier,
) {
    val uiState by viewModel.uiState.collectAsState()
    val activePanel by viewModel.activePanel.collectAsState()
    val defaultHighlightColor by viewModel.defaultHighlightColor.collectAsState()
    val playbackState by viewModel.playbackState.collectAsState()
    val snackbarHostState = remember { SnackbarHostState() }
    val coroutineScope = rememberCoroutineScope()
    val isDarkMode = isSystemInDarkTheme()

    var selectedText by remember { mutableStateOf<SelectedTextInfo?>(null) }
    var tappedHighlight by remember { mutableStateOf<HighlightData?>(null) }
    var tagSheetHighlightId by remember { mutableStateOf<String?>(null) }
    var availableTags by remember { mutableStateOf<List<TagData>>(emptyList()) }
    var scrollToPercent by remember { mutableStateOf<Float?>(null) }
    var anchorScroll by remember { mutableStateOf<AnchorScrollRequest?>(null) }
    val chromeState = remember { ReaderChromeState() }
    // The in-reader Mila session is created on first open and kept alive (even while
    // the drawer is closed) so the conversation persists for the rest of the read.
    var milaStarted by remember { mutableStateOf(false) }

    val chromeSuppressed =
        isChromeSuppressed(
            activePanel = activePanel,
            hasSelection = selectedText != null,
            hasTappedHighlight = tappedHighlight != null,
            hasOpenTagSheet = tagSheetHighlightId != null,
        )
    ReaderChromeSuppressionEffect(chromeState, chromeSuppressed)

    val handleScrollProgress: (Float, Float) -> Unit = { percent, scrollTop ->
        chromeState.onScroll(scrollTop, suppressed = chromeSuppressed)
        viewModel.onScrollProgress(percent)
    }

    // The whole screen belongs to the page for as long as the reader is open — the
    // system bars do not come back when the reader's own controls do. Retiring the
    // furniture and giving up the screen are separate ideas; tying them together made
    // the page resize under the reader every time the controls came and went.
    ImmersiveSystemBars(hidden = true)

    LaunchedEffect(viewModel) {
        viewModel.effects.collect { effect ->
            when (effect) {
                is ReaderEffect.NavigateBack -> onNavigateBack()
                is ReaderEffect.ShowSnackbar -> snackbarHostState.showSnackbar(effect.message)
                is ReaderEffect.ScrollToPercent -> scrollToPercent = effect.percent
                is ReaderEffect.ScrollToAnchor ->
                    anchorScroll =
                        AnchorScrollRequest(
                            id = effect.id,
                            fallbackIndex = effect.fallbackIndex,
                            nonce = (anchorScroll?.nonce ?: 0L) + 1L,
                        )
            }
        }
    }

    LaunchedEffect(activePanel) {
        if (activePanel == DataPanel.NOTE || activePanel == DataPanel.INFO) {
            viewModel.loadTagsForPicker { availableTags = it }
        }
        if (activePanel == DataPanel.MILA) {
            milaStarted = true
        }
    }

    Scaffold(
        modifier = modifier,
        snackbarHost = { SnackbarHost(snackbarHostState) },
    ) { paddingValues ->
        when (val state = uiState) {
            is ReaderUiState.Loading -> ReaderLoadingState(Modifier.padding(paddingValues))
            is ReaderUiState.Error -> ReaderErrorState(state.message, Modifier.padding(paddingValues))
            is ReaderUiState.Success -> {
                ReaderSuccessContent(
                    state = state,
                    activePanel = activePanel,
                    defaultHighlightColor = defaultHighlightColor,
                    playbackState = playbackState,
                    availableTags = availableTags,
                    immersive = chromeState.immersive,
                    selectedText = selectedText,
                    tappedHighlight = tappedHighlight,
                    tagSheetHighlightId = tagSheetHighlightId,
                    milaStarted = milaStarted,
                    scrollToPercent = scrollToPercent,
                    anchorScroll = anchorScroll,
                    isDarkMode = isDarkMode,
                    snackbarHostState = snackbarHostState,
                    coroutineScope = coroutineScope,
                    viewModel = viewModel,
                    milaViewModelProvider = milaViewModelProvider,
                    onNavigateToAiSettings = onNavigateToAiSettings,
                    onNavigateToItem = onNavigateToItem,
                    // Selecting text or tapping a highlight is a deliberate reach for the
                    // chrome. Wiring reveal here gives iOS a second path that does not
                    // depend on the synthetic tap surviving WKWebView's gesture recognisers.
                    onSelectedTextChanged = {
                        if (it != null) chromeState.reveal()
                        selectedText = it
                    },
                    onTappedHighlightChanged = {
                        if (it != null) chromeState.reveal()
                        tappedHighlight = it
                    },
                    onTagSheetHighlightIdChanged = { tagSheetHighlightId = it },
                    onAvailableTagsChanged = { availableTags = it },
                    onScrollToPercentConsumed = {
                        viewModel.onScrollRestored()
                        scrollToPercent = null
                    },
                    onAnchorScrollConsumed = { anchorScroll = null },
                    handleScrollProgress = handleScrollProgress,
                    onReaderTap = { chromeState.reveal() },
                )
            }
        }
    }
}

@Composable
private fun ReaderLoadingState(modifier: Modifier = Modifier) {
    Box(
        modifier = modifier.fillMaxSize(),
        contentAlignment = Alignment.Center,
    ) {
        CircularProgressIndicator()
    }
}

@Composable
private fun ReaderErrorState(
    message: String,
    modifier: Modifier = Modifier,
) {
    Box(
        modifier = modifier.fillMaxSize(),
        contentAlignment = Alignment.Center,
    ) {
        Text(
            text = message,
            style = MaterialTheme.typography.bodyLarge,
            color = MaterialTheme.colorScheme.error,
        )
    }
}
