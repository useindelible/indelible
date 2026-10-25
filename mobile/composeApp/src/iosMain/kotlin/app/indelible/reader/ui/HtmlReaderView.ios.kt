@file:OptIn(kotlinx.cinterop.ExperimentalForeignApi::class)

package app.indelible.reader.ui

import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.interop.UIKitView
import app.indelible.reader.model.HighlightData
import app.indelible.reader.model.ReaderPreferences
import kotlinx.cinterop.ExperimentalForeignApi
import kotlinx.serialization.json.Json
import platform.UIKit.UIApplication
import platform.UIKit.UIColor
import platform.UIKit.UIScrollViewContentInsetAdjustmentBehavior
import platform.WebKit.WKNavigation
import platform.WebKit.WKNavigationAction
import platform.WebKit.WKNavigationActionPolicy
import platform.WebKit.WKNavigationDelegateProtocol
import platform.WebKit.WKScriptMessage
import platform.WebKit.WKScriptMessageHandlerProtocol
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
    var pageLoaded by remember { mutableStateOf(false) }
    val messageHandler = remember { IosMessageHandler(callbacks) }
    val navigationDelegate =
        remember {
            IosNavigationDelegate {
                pageLoaded = true
                latestOnContentLoaded.value()
            }
        }
    var webViewRef by remember { mutableStateOf<WKWebView?>(null) }

    UIKitView(
        factory = { readerWebView(messageHandler, navigationDelegate, fullHtml).also { webViewRef = it } },
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
    ) { webViewRef?.evaluateJavaScript(it, null) }
}

private val BRIDGE_MESSAGES =
    listOf(
        "onScroll",
        "onTextSelected",
        "onSelectionCleared",
        "onHighlightTapped",
        "onSummaryAction",
        "onReaderTap",
        "onPositioned",
        "onBookLink",
    )

private val BRIDGE_SCRIPT =
    """
    window.NativeBridge = {
        onScroll: function(percent, scrollTop, offset, atEnd) {
            window.webkit.messageHandlers.onScroll.postMessage(
                JSON.stringify({percent: percent, scrollTop: scrollTop, offset: offset, atEnd: atEnd})
            );
        },
        onPositioned: function() {
            window.webkit.messageHandlers.onPositioned.postMessage("");
        },
        onBookLink: function(href) {
            window.webkit.messageHandlers.onBookLink.postMessage(href);
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

private fun readerWebView(
    messageHandler: IosMessageHandler,
    navigationDelegate: IosNavigationDelegate,
    fullHtml: String,
): WKWebView {
    val config = WKWebViewConfiguration()
    val controller = config.userContentController
    BRIDGE_MESSAGES.forEach { controller.addScriptMessageHandler(messageHandler, it) }
    // A document-start user script is not subject to the page's CSP, so the reader's strict
    // script-src can block injected inline scripts while the bridge still loads.
    controller.addUserScript(
        WKUserScript(
            source = BRIDGE_SCRIPT,
            injectionTime = WKUserScriptInjectionTime.WKUserScriptInjectionTimeAtDocumentStart,
            forMainFrameOnly = true,
        ),
    )
    val webView = WKWebView(frame = kotlinx.cinterop.cValue { }, configuration = config)
    webView.navigationDelegate = navigationDelegate
    // Transparent so the native aura layer behind the WebView shows through the reader canvas.
    webView.opaque = false
    webView.backgroundColor = UIColor.clearColor()
    webView.scrollView.opaque = false
    webView.scrollView.backgroundColor = UIColor.clearColor()
    // The HTML's own padding-top (from the Compose insets) is the single source of top spacing.
    webView.scrollView.contentInsetAdjustmentBehavior =
        UIScrollViewContentInsetAdjustmentBehavior.UIScrollViewContentInsetAdjustmentNever
    webView.loadHTMLString(fullHtml, baseURL = null)
    return webView
}

private class IosMessageHandler(
    private val callbacks: ReaderBridgeCallbacks,
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
                    callbacks.onScroll(ScrollSample(data.percent, data.scrollTop, data.offset, data.atEnd))
                } catch (_: Exception) {
                }
            }
            "onTextSelected" -> {
                try {
                    val data = json.decodeFromString<TextSelectionMessage>(body)
                    val rect = parseRectString(data.rect)
                    callbacks.onTextSelected(data.text, data.startOffset, data.endOffset, rect)
                } catch (_: Exception) {
                }
            }
            "onSelectionCleared" -> callbacks.onSelectionCleared()
            "onReaderTap" -> callbacks.onReaderTap()
            "onHighlightTapped" -> {
                try {
                    val data = json.decodeFromString<HighlightTapMessage>(body)
                    val rect = parseRectString(data.rect)
                    callbacks.onHighlightTapped(data.highlightId, rect)
                } catch (_: Exception) {
                }
            }
            "onSummaryAction" -> callbacks.onSummaryAction(body)
            "onPositioned" -> callbacks.onPositioned()
            "onBookLink" -> callbacks.onBookLink(body)
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
    val offset: Int = -1,
    val atEnd: Boolean = false,
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
