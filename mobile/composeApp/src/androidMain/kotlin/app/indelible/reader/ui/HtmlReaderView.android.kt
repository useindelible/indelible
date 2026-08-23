package app.indelible.reader.ui

import android.annotation.SuppressLint
import android.content.Intent
import android.view.ActionMode
import android.view.Menu
import android.view.MenuItem
import android.webkit.JavascriptInterface
import android.webkit.WebResourceRequest
import android.webkit.WebView
import android.webkit.WebViewClient
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.setValue
import androidx.compose.runtime.snapshotFlow
import androidx.compose.ui.Modifier
import androidx.compose.ui.viewinterop.AndroidView
import app.indelible.reader.model.HighlightData
import app.indelible.reader.model.ReaderPreferences
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.serialization.json.Json

@SuppressLint("SetJavaScriptEnabled")
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
    // rememberUpdatedState ensures the JS interface always invokes the latest lambdas
    // even though the interface itself is created once via remember.
    val latestOnScrollProgress = rememberUpdatedState(onScrollProgress)
    val latestOnTextSelected = rememberUpdatedState(onTextSelected)
    val latestOnSelectionCleared = rememberUpdatedState(onSelectionCleared)
    val latestOnHighlightTapped = rememberUpdatedState(onHighlightTapped)
    val latestOnSummaryAction = rememberUpdatedState(onSummaryAction)
    val latestOnReaderTap = rememberUpdatedState(onReaderTap)

    val jsInterface =
        remember {
            ReaderJsInterface(
                onScroll = { p, s -> latestOnScrollProgress.value(p, s) },
                onTextSelected = { t, s, e, r -> latestOnTextSelected.value(t, s, e, r) },
                onSelectionCleared = { latestOnSelectionCleared.value() },
                onHighlightTapped = { id, r -> latestOnHighlightTapped.value(id, r) },
                onSummaryAction = { latestOnSummaryAction.value(it) },
                onReaderTap = { latestOnReaderTap.value() },
            )
        }

    val fullHtml =
        remember(
            htmlContent,
            preferences,
            highlights,
            isDarkMode,
            localization,
            summaryHtml,
            summaryPoints,
            topContentPaddingPx,
            artwork,
        ) {
            ReaderHtmlTemplate.build(
                articleHtml = htmlContent,
                preferences = preferences,
                highlights = highlights,
                localization = localization,
                isDarkMode = isDarkMode,
                articleTitle = articleTitle,
                articleAuthor = articleAuthor,
                articleDomain = articleDomain,
                summaryHtml = summaryHtml,
                summaryPoints = summaryPoints,
                topContentPaddingPx = topContentPaddingPx,
                artwork = artwork,
            )
        }

    var webViewRef by remember { mutableStateOf<WebView?>(null) }

    AndroidView(
        factory = { context ->
            object : WebView(context) {
                // Suppress Android's native text-selection toolbar (Copy / Select All / Share);
                // it floats over the reader's own highlight pill. Selection and drag handles are
                // kept alive, so the JS selectionchange bridge still drives our pill.
                override fun startActionMode(callback: ActionMode.Callback?): ActionMode? =
                    super.startActionMode(SuppressingActionModeCallback())

                override fun startActionMode(
                    callback: ActionMode.Callback?,
                    type: Int,
                ): ActionMode? = super.startActionMode(SuppressingActionModeCallback(), type)
            }.apply {
                settings.javaScriptEnabled = true
                settings.domStorageEnabled = true
                settings.loadWithOverviewMode = true
                settings.useWideViewPort = true
                // Transparent so the native aura layer behind the WebView bleeds
                // through the reader canvas (painted at < 1.0 alpha in the template).
                setBackgroundColor(android.graphics.Color.TRANSPARENT)
                addJavascriptInterface(jsInterface, "NativeBridge")
                webViewClient =
                    object : WebViewClient() {
                        override fun shouldOverrideUrlLoading(
                            view: WebView?,
                            request: WebResourceRequest?,
                        ): Boolean {
                            val req = request ?: return false
                            // Subframe loads (the YouTube embed iframe) proceed; only top-level
                            // navigation is intercepted so injected content cannot redirect the
                            // reader in-app. Real web links open in the system browser instead.
                            if (!req.isForMainFrame) return false
                            val target = req.url
                            if (target?.scheme == "http" || target?.scheme == "https") {
                                runCatching {
                                    view?.context?.startActivity(Intent(Intent.ACTION_VIEW, target))
                                }
                            }
                            return true
                        }

                        override fun onPageFinished(
                            view: WebView?,
                            url: String?,
                        ) {
                            super.onPageFinished(view, url)
                            onContentLoaded()
                        }
                    }
                loadDataWithBaseURL(null, fullHtml, "text/html", "utf-8", null)
                webViewRef = this
            }
        },
        update = { webView ->
            webViewRef = webView
        },
        modifier = modifier,
    )

    LaunchedEffect(scrollToPercent) {
        val percent = scrollToPercent ?: return@LaunchedEffect
        webViewRef?.evaluateJavascript("window.scrollToPercent($percent);", null)
        onScrollRestored()
    }

    LaunchedEffect(anchorScroll) {
        val anchor = anchorScroll ?: return@LaunchedEffect
        val escapedId = anchor.id.replace("\\", "\\\\").replace("'", "\\'")
        webViewRef?.evaluateJavascript(
            "window.scrollToAnchor('$escapedId', ${anchor.fallbackIndex});",
            null,
        )
        onAnchorScrolled()
    }

    LaunchedEffect(preferences, isDarkMode) {
        snapshotFlow { preferences }
            .distinctUntilChanged()
            .collect { prefs ->
                val css = ReaderHtmlTemplate.buildTypographyCss(prefs, isDarkMode)
                val escaped =
                    css
                        .replace("\\", "\\\\")
                        .replace("'", "\\'")
                        .replace("\n", "\\n")
                val colorScheme = ReaderHtmlTemplate.colorSchemeFor(prefs, isDarkMode)
                webViewRef?.evaluateJavascript(
                    "window.updateTypography('$escaped', '$colorScheme');",
                    null,
                )
            }
    }

    LaunchedEffect(highlights) {
        snapshotFlow { highlights }
            .distinctUntilChanged()
            .collect { hl ->
                val json = ReaderHtmlTemplate.buildHighlightJson(hl)
                webViewRef?.evaluateJavascript(
                    "window.applyHighlights($json);",
                    null,
                )
            }
    }

    LaunchedEffect(speakingSentenceIndex) {
        webViewRef?.evaluateJavascript("window.setSpeaking($speakingSentenceIndex);", null)
    }

    LaunchedEffect(immersive) {
        webViewRef?.evaluateJavascript("window.setReaderImmersive($immersive);", null)
    }
}

private class ReaderJsInterface(
    private val onScroll: (Float, Float) -> Unit,
    private val onTextSelected: (String, Int, Int, SelectionRect) -> Unit,
    private val onSelectionCleared: () -> Unit,
    private val onHighlightTapped: (String, SelectionRect) -> Unit,
    private val onSummaryAction: (String) -> Unit,
    private val onReaderTap: () -> Unit,
) {
    private val json = Json { ignoreUnknownKeys = true }

    @JavascriptInterface
    fun onScroll(
        percent: Double,
        scrollTop: Double,
    ) {
        onScroll(percent.toFloat(), scrollTop.toFloat())
    }

    @JavascriptInterface
    fun onSummaryAction(action: String) {
        // .invoke() targets the constructor property, not this same-named method.
        onSummaryAction.invoke(action)
    }

    @JavascriptInterface
    fun onTextSelected(
        text: String,
        startOffset: Int,
        endOffset: Int,
        rectJson: String,
    ) {
        val rect = parseRect(rectJson)
        onTextSelected(text, startOffset, endOffset, rect)
    }

    @JavascriptInterface
    fun onSelectionCleared() {
        onSelectionCleared.invoke()
    }

    @JavascriptInterface
    fun onReaderTap() {
        onReaderTap.invoke()
    }

    @JavascriptInterface
    fun onHighlightTapped(
        highlightId: String,
        rectJson: String,
    ) {
        val rect = parseRect(rectJson)
        onHighlightTapped(highlightId, rect)
    }

    private fun parseRect(rectJson: String): SelectionRect =
        try {
            val obj = json.decodeFromString<RectData>(rectJson)
            SelectionRect(obj.x, obj.y, obj.width, obj.height)
        } catch (_: Exception) {
            SelectionRect(0f, 0f, 0f, 0f)
        }
}

@kotlinx.serialization.Serializable
private data class RectData(
    val x: Float,
    val y: Float,
    val width: Float,
    val height: Float,
)

/**
 * Keeps the text-selection ActionMode alive (so the selection and its drag handles persist)
 * while contributing no menu items, which prevents the native floating selection toolbar from
 * rendering over the reader's own highlight pill.
 */
private class SuppressingActionModeCallback : ActionMode.Callback {
    override fun onCreateActionMode(
        mode: ActionMode?,
        menu: Menu?,
    ): Boolean = true

    override fun onPrepareActionMode(
        mode: ActionMode?,
        menu: Menu?,
    ): Boolean {
        menu?.clear()
        return true
    }

    override fun onActionItemClicked(
        mode: ActionMode?,
        item: MenuItem?,
    ): Boolean = false

    override fun onDestroyActionMode(mode: ActionMode?) {}
}
