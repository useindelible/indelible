package app.indelible.reader.ui

import android.annotation.SuppressLint
import android.content.Context
import android.content.Intent
import android.os.Handler
import android.os.Looper
import android.view.ActionMode
import android.view.Menu
import android.view.MenuItem
import android.webkit.JavascriptInterface
import android.webkit.WebResourceRequest
import android.webkit.WebView
import android.webkit.WebViewClient
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.viewinterop.AndroidView
import app.indelible.reader.model.HighlightData
import app.indelible.reader.model.ReaderPreferences
import kotlinx.serialization.json.Json

@Composable
actual fun HtmlReaderView(
    htmlContent: String,
    highlights: List<HighlightData>,
    preferences: ReaderPreferences,
    isDarkMode: Boolean,
    scrollToPercent: Float?,
    anchorScroll: AnchorScrollRequest?,
    onScrollProgress: (ScrollSample) -> Unit,
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
    assets: ReaderAssets,
    scrollToCharOffset: CharOffsetRequest?,
    captureRequest: Long?,
    clearSelection: Long?,
    onPositioned: () -> Unit,
    holdUntilPositioned: Boolean,
    onBookLink: (String) -> Unit,
    modifier: Modifier,
) {
    val callbacks =
        rememberReaderBridgeCallbacks(
            onScrollProgress,
            onTextSelected,
            onSelectionCleared,
            onHighlightTapped,
            onSummaryAction,
            onReaderTap,
            onPositioned,
            onBookLink,
        )
    val jsInterface = remember { ReaderJsInterface(callbacks) }
    val options =
        ReaderPageOptions(
            localization,
            ReaderArticleMeta(articleTitle, articleAuthor, articleDomain),
            ReaderSummary(summaryHtml, summaryPoints),
            topContentPaddingPx,
            assets,
            holdUntilPositioned,
        )
    val fullHtml =
        remember(htmlContent, preferences, highlights, options) {
            ReaderHtmlTemplate.build(htmlContent, preferences, highlights, options)
        }
    val latestOnContentLoaded = rememberUpdatedState(onContentLoaded)
    var webViewRef by remember { mutableStateOf<WebView?>(null) }
    var pageLoaded by remember { mutableStateOf(false) }

    AndroidView(
        factory = { context ->
            readerWebView(context, jsInterface, fullHtml) {
                pageLoaded = true
                latestOnContentLoaded.value()
            }.also { webViewRef = it }
        },
        update = { webView -> webViewRef = webView },
        modifier = modifier,
    )

    ReaderBridgeEffects(
        ReaderBridgeInputs(
            pageLoaded,
            scrollToPercent,
            anchorScroll,
            preferences,
            highlights,
            speakingSentenceIndex,
            immersive,
            scrollToCharOffset,
            captureRequest,
            clearSelection,
        ),
        onScrollRestored,
        onAnchorScrolled,
    ) { webViewRef?.evaluateJavascript(it, null) }
}

@SuppressLint("SetJavaScriptEnabled")
private fun readerWebView(
    context: Context,
    jsInterface: ReaderJsInterface,
    fullHtml: String,
    onPageFinished: () -> Unit,
): WebView =
    object : WebView(context) {
        // Android's own selection toolbar (Copy / Select All / Share) would float over the reader's
        // highlight pill; the selection and its drag handles stay alive without it.
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
        // Transparent so the native aura layer behind the WebView shows through the reader canvas.
        setBackgroundColor(android.graphics.Color.TRANSPARENT)
        addJavascriptInterface(jsInterface, "NativeBridge")
        webViewClient = ReaderWebViewClient(onPageFinished)
        loadDataWithBaseURL(null, fullHtml, "text/html", "utf-8", null)
    }

private class ReaderWebViewClient(
    private val onPageFinished: () -> Unit,
) : WebViewClient() {
    override fun shouldOverrideUrlLoading(
        view: WebView?,
        request: WebResourceRequest?,
    ): Boolean {
        val req = request ?: return false
        val decision =
            ReaderNavigationPolicy.decide(
                isMainFrame = req.isForMainFrame,
                scheme = req.url?.scheme,
                initialReaderDocumentPending = false,
            )
        if (decision == ReaderNavigationDecision.OpenExternally) {
            runCatching { view?.context?.startActivity(Intent(Intent.ACTION_VIEW, req.url)) }
        }
        return decision != ReaderNavigationDecision.Allow
    }

    override fun onPageFinished(
        view: WebView?,
        url: String?,
    ) {
        super.onPageFinished(view, url)
        onPageFinished()
    }
}

private class ReaderJsInterface(
    private val callbacks: ReaderBridgeCallbacks,
) {
    private val json = Json { ignoreUnknownKeys = true }

    // WebView calls these on its own bridge thread; the reader's state is only touched on the main thread.
    private val main = Handler(Looper.getMainLooper())

    private fun onMain(block: () -> Unit) {
        main.post(block)
    }

    @JavascriptInterface
    fun onScroll(
        percent: Double,
        scrollTop: Double,
        offset: Double,
        atEnd: Boolean,
    ) = onMain { callbacks.onScroll(ScrollSample(percent.toFloat(), scrollTop.toFloat(), offset.toInt(), atEnd)) }

    @JavascriptInterface
    fun onPositioned() = onMain { callbacks.onPositioned() }

    @JavascriptInterface
    fun onBookLink(href: String) = onMain { callbacks.onBookLink(href) }

    @JavascriptInterface
    fun onSummaryAction(action: String) = onMain { callbacks.onSummaryAction(action) }

    @JavascriptInterface
    fun onTextSelected(
        text: String,
        startOffset: Int,
        endOffset: Int,
        rectJson: String,
    ) {
        val rect = parseRect(rectJson)
        onMain { callbacks.onTextSelected(text, startOffset, endOffset, rect) }
    }

    @JavascriptInterface
    fun onSelectionCleared() = onMain { callbacks.onSelectionCleared() }

    @JavascriptInterface
    fun onReaderTap() = onMain { callbacks.onReaderTap() }

    @JavascriptInterface
    fun onHighlightTapped(
        highlightId: String,
        rectJson: String,
    ) {
        val rect = parseRect(rectJson)
        onMain { callbacks.onHighlightTapped(highlightId, rect) }
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

/** Keeps the selection ActionMode alive while contributing no menu items. */
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

    override fun onDestroyActionMode(mode: ActionMode?) = Unit
}
