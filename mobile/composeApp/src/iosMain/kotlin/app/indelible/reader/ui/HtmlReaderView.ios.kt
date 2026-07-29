@file:OptIn(kotlinx.cinterop.ExperimentalForeignApi::class)

package app.indelible.reader.ui

import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.runtime.snapshotFlow
import androidx.compose.ui.Modifier
import androidx.compose.ui.interop.UIKitView
import app.indelible.reader.model.HighlightData
import app.indelible.reader.model.ReaderPreferences
import kotlinx.cinterop.ExperimentalForeignApi
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.datetime.Instant
import kotlinx.serialization.json.Json
import platform.WebKit.WKNavigation
import platform.WebKit.WKNavigationAction
import platform.WebKit.WKNavigationActionPolicy
import platform.WebKit.WKNavigationDelegateProtocol
import platform.WebKit.WKScriptMessage
import platform.WebKit.WKScriptMessageHandlerProtocol
import platform.UIKit.UIApplication
import platform.UIKit.UIColor
import platform.UIKit.UIScrollViewContentInsetAdjustmentBehavior
import platform.WebKit.WKUserContentController
import platform.WebKit.WKUserScript
import platform.WebKit.WKUserScriptInjectionTime
import platform.WebKit.WKWebView
import platform.WebKit.WKWebViewConfiguration
import platform.darwin.NSObject

@OptIn(ExperimentalForeignApi::class)
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
    articlePublishedAt: Instant?,
    articleReadingTimeMinutes: Int?,
    summaryHtml: String?,
    summaryPoints: List<String>,
    onSummaryAction: (String) -> Unit,
    speakingSentenceIndex: Int,
    immersive: Boolean,
    topContentPaddingPx: Int,
    artwork: LoadedReaderArtwork?,
    modifier: Modifier,
) {
    val fullHtml =
        remember(
            htmlContent,
            preferences,
            highlights,
            isDarkMode,
            summaryHtml,
            summaryPoints,
            topContentPaddingPx,
            artwork,
        ) {
            ReaderHtmlTemplate.build(
                htmlContent,
                preferences,
                highlights,
                isDarkMode,
                articleTitle,
                articleAuthor,
                articleDomain,
                articlePublishedAt,
                articleReadingTimeMinutes,
                summaryHtml,
                summaryPoints,
                topContentPaddingPx,
                artwork,
            )
        }

    val messageHandler =
        remember {
            IosMessageHandler(
                onScroll = onScrollProgress,
                onTextSelected = onTextSelected,
                onSelectionCleared = onSelectionCleared,
                onHighlightTapped = onHighlightTapped,
                onSummaryAction = onSummaryAction,
                onReaderTap = onReaderTap,
            )
        }

    val navigationDelegate =
        remember {
            IosNavigationDelegate(onContentLoaded)
        }

    var webViewRef by remember { mutableStateOf<WKWebView?>(null) }

    UIKitView(
        factory = {
            val config = WKWebViewConfiguration()
            val controller = config.userContentController

            val bridgeScript =
                """
                window.NativeBridge = {
                    onScroll: function(percent, scrollTop) {
                        window.webkit.messageHandlers.onScroll.postMessage(
                            JSON.stringify({percent: percent, scrollTop: scrollTop})
                        );
                    },
                    onTextSelected: function(text, startOffset, endOffset, rectJson) {
                        window.webkit.messageHandlers.onTextSelected.postMessage(
                            JSON.stringify({text: text, startOffset: startOffset, endOffset: endOffset, rect: rectJson})
                        );
                    },
                    onSelectionCleared: function() {
                        window.webkit.messageHandlers.onSelectionCleared.postMessage("");
                    },
                    onHighlightTapped: function(highlightId, rectJson) {
                        window.webkit.messageHandlers.onHighlightTapped.postMessage(
                            JSON.stringify({highlightId: highlightId, rect: rectJson})
                        );
                    },
                    onSummaryAction: function(action) {
                        window.webkit.messageHandlers.onSummaryAction.postMessage(action);
                    },
                    onReaderTap: function() {
                        window.webkit.messageHandlers.onReaderTap.postMessage("");
                    }
                };
                """.trimIndent()

            controller.addScriptMessageHandler(messageHandler, "onScroll")
            controller.addScriptMessageHandler(messageHandler, "onTextSelected")
            controller.addScriptMessageHandler(messageHandler, "onSelectionCleared")
            controller.addScriptMessageHandler(messageHandler, "onHighlightTapped")
            controller.addScriptMessageHandler(messageHandler, "onSummaryAction")
            controller.addScriptMessageHandler(messageHandler, "onReaderTap")

            // Inject the native bridge as a document-start user script rather than a page
            // <script>. User-script injection is not subject to the document CSP, so the reader's
            // strict script-src can block injected inline scripts while the bridge still loads.
            controller.addUserScript(
                WKUserScript(
                    source = bridgeScript,
                    injectionTime = WKUserScriptInjectionTime.WKUserScriptInjectionTimeAtDocumentStart,
                    forMainFrameOnly = true,
                ),
            )

            val webView = WKWebView(frame = kotlinx.cinterop.cValue { }, configuration = config)
            webView.navigationDelegate = navigationDelegate
            // Transparent so the native aura layer behind the WebView bleeds
            // through the reader canvas (painted at < 1.0 alpha in the template).
            webView.opaque = false
            webView.backgroundColor = UIColor.clearColor()
            webView.scrollView.opaque = false
            webView.scrollView.backgroundColor = UIColor.clearColor()
            // Disable WKWebView's automatic safe-area inset so the HTML's own
            // padding-top (passed from Compose insets) is the single source of top
            // spacing; the aura then paints from the true top behind the status bar.
            webView.scrollView.contentInsetAdjustmentBehavior =
                UIScrollViewContentInsetAdjustmentBehavior.UIScrollViewContentInsetAdjustmentNever

            webView.loadHTMLString(fullHtml, baseURL = null)
            webViewRef = webView
            webView
        },
        modifier = modifier,
    )

    LaunchedEffect(scrollToPercent) {
        val percent = scrollToPercent ?: return@LaunchedEffect
        webViewRef?.evaluateJavaScript("window.scrollToPercent($percent);", null)
        onScrollRestored()
    }

    LaunchedEffect(anchorScroll) {
        val anchor = anchorScroll ?: return@LaunchedEffect
        val escapedId = anchor.id.replace("\\", "\\\\").replace("'", "\\'")
        webViewRef?.evaluateJavaScript(
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
                webViewRef?.evaluateJavaScript("window.updateTypography('$escaped', '$colorScheme');", null)
            }
    }

    LaunchedEffect(highlights) {
        snapshotFlow { highlights }
            .distinctUntilChanged()
            .collect { hl ->
                val json = ReaderHtmlTemplate.buildHighlightJson(hl)
                webViewRef?.evaluateJavaScript("window.applyHighlights($json);", null)
            }
    }

    LaunchedEffect(speakingSentenceIndex) {
        webViewRef?.evaluateJavaScript("window.setSpeaking($speakingSentenceIndex);", null)
    }

    LaunchedEffect(immersive) {
        webViewRef?.evaluateJavaScript("window.setReaderImmersive($immersive);", null)
    }
}

private class IosMessageHandler(
    private val onScroll: (Float, Float) -> Unit,
    private val onTextSelected: (String, Int, Int, SelectionRect) -> Unit,
    private val onSelectionCleared: () -> Unit,
    private val onHighlightTapped: (String, SelectionRect) -> Unit,
    private val onSummaryAction: (String) -> Unit,
    private val onReaderTap: () -> Unit,
) : NSObject(),
    WKScriptMessageHandlerProtocol {
    private val json = Json { ignoreUnknownKeys = true }

    override fun userContentController(
        userContentController: WKUserContentController,
        didReceiveScriptMessage: WKScriptMessage,
    ) {
        val name = didReceiveScriptMessage.name
        val body = didReceiveScriptMessage.body?.toString() ?: return

        when (name) {
            "onScroll" -> {
                try {
                    val data = json.decodeFromString<ScrollMessage>(body)
                    onScroll(data.percent, data.scrollTop)
                } catch (_: Exception) {
                }
            }
            "onTextSelected" -> {
                try {
                    val data = json.decodeFromString<TextSelectionMessage>(body)
                    val rect = parseRectString(data.rect)
                    onTextSelected(data.text, data.startOffset, data.endOffset, rect)
                } catch (_: Exception) {
                }
            }
            "onSelectionCleared" -> onSelectionCleared()
            "onReaderTap" -> onReaderTap()
            "onHighlightTapped" -> {
                try {
                    val data = json.decodeFromString<HighlightTapMessage>(body)
                    val rect = parseRectString(data.rect)
                    onHighlightTapped(data.highlightId, rect)
                } catch (_: Exception) {
                }
            }
            "onSummaryAction" -> onSummaryAction(body)
        }
    }

    private fun parseRectString(rectJson: String): SelectionRect =
        try {
            val obj = json.decodeFromString<RectPayload>(rectJson)
            SelectionRect(obj.x, obj.y, obj.width, obj.height)
        } catch (_: Exception) {
            SelectionRect(0f, 0f, 0f, 0f)
        }
}

private class IosNavigationDelegate(
    private val onContentLoaded: () -> Unit,
) : NSObject(),
    WKNavigationDelegateProtocol {
    private var initialReaderDocumentPending = true

    override fun webView(
        webView: WKWebView,
        decidePolicyForNavigationAction: WKNavigationAction,
        decisionHandler: (WKNavigationActionPolicy) -> Unit,
    ) {
        val action = decidePolicyForNavigationAction
        val isMainFrame = action.targetFrame?.isMainFrame() ?: true
        val url = action.request.URL
        when (
            ReaderNavigationPolicy.decide(
                isMainFrame = isMainFrame,
                scheme = url?.scheme,
                initialReaderDocumentPending = initialReaderDocumentPending,
            )
        ) {
            ReaderNavigationDecision.Allow -> {
                decisionHandler(WKNavigationActionPolicy.WKNavigationActionPolicyAllow)
            }
            ReaderNavigationDecision.AllowInitialDocument -> {
                initialReaderDocumentPending = false
                decisionHandler(WKNavigationActionPolicy.WKNavigationActionPolicyAllow)
            }
            ReaderNavigationDecision.OpenExternally -> {
                if (url != null) {
                    UIApplication.sharedApplication.openURL(url)
                }
                decisionHandler(WKNavigationActionPolicy.WKNavigationActionPolicyCancel)
            }
            ReaderNavigationDecision.Cancel -> {
                decisionHandler(WKNavigationActionPolicy.WKNavigationActionPolicyCancel)
            }
        }
    }

    override fun webView(
        webView: WKWebView,
        didFinishNavigation: WKNavigation?,
    ) {
        initialReaderDocumentPending = false
        onContentLoaded()
    }
}

@kotlinx.serialization.Serializable
private data class ScrollMessage(
    val percent: Float,
    val scrollTop: Float,
)

@kotlinx.serialization.Serializable
private data class TextSelectionMessage(
    val text: String,
    val startOffset: Int,
    val endOffset: Int,
    val rect: String,
)

@kotlinx.serialization.Serializable
private data class HighlightTapMessage(
    val highlightId: String,
    val rect: String,
)

@kotlinx.serialization.Serializable
private data class RectPayload(
    val x: Float,
    val y: Float,
    val width: Float,
    val height: Float,
)
