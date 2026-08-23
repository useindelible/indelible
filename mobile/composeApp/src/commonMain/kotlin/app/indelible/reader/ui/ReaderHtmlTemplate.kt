package app.indelible.reader.ui

import app.indelible.reader.model.HighlightData
import app.indelible.reader.model.ReaderBackground
import app.indelible.reader.model.ReaderPreferences
import kotlin.random.Random

object ReaderHtmlTemplate {
    // Mirrors the base top gap in READER_RESET_CSS's `body { padding: 26px ... }`. The body's
    // inline padding-top is (native chrome height + this) so the masthead clears the
    // status bar and top bar that overlay the full-bleed WebView, while the aura
    // still paints from the very top behind them.
    private const val READER_MASTHEAD_TOP_GAP_PX = 26

    // Used when a document renders without a drawing (video, which carries a native
    // poster instead). The veils still need a finite fade range.
    private const val DEFAULT_VEIL_RANGE = 60

    // Per-render CSP nonce: the reader's own inline bridge script carries it so the strict
    // script-src can run it while blocking any injected inline script or event-handler attribute.
    private fun randomNonce(): String =
        Random.nextBytes(16).joinToString("") {
            (it.toInt() and 0xFF).toString(16).padStart(2, '0')
        }

    fun build(
        articleHtml: String,
        preferences: ReaderPreferences,
        highlights: List<HighlightData>,
        localization: ReaderHtmlLocalization =
            ReaderHtmlLocalization(
                publishedDate = null,
                readingTime = null,
                summaryLabel = "",
                askFollowUpLabel = "",
            ),
        isDarkMode: Boolean = false,
        articleTitle: String = "",
        articleAuthor: String? = null,
        articleDomain: String? = null,
        summaryHtml: String? = null,
        summaryPoints: List<String> = emptyList(),
        topContentPaddingPx: Int = 0,
        artwork: LoadedReaderArtwork? = null,
    ): String {
        val css = buildCss(preferences, isDarkMode)
        val veilRange = artwork?.artwork?.veilRange ?: DEFAULT_VEIL_RANGE
        val cspNonce = randomNonce()
        val colorScheme = colorSchemeFor(preferences, isDarkMode)
        val highlightJson = buildHighlightJson(highlights)
        val mastheadHtml =
            buildMasthead(
                ReaderMastheadData(
                    title = articleTitle,
                    author = articleAuthor,
                    domain = articleDomain,
                    hasSummary = summaryHtml != null,
                ),
                localization = localization,
            )
        val summaryBlockHtml = buildSummaryBlock(summaryHtml, summaryPoints, localization.askFollowUpLabel)
        return """
<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<meta http-equiv="Content-Security-Policy" content="default-src 'none'; script-src 'nonce-$cspNonce'; style-src 'unsafe-inline' https:; font-src https: data:; img-src https: http: data:; media-src https: http: data:; frame-src https://www.youtube.com https://www.youtube-nocookie.com; connect-src 'none'; base-uri 'none'; form-action 'none'">
<meta name="viewport" content="width=device-width, initial-scale=1, maximum-scale=1, user-scalable=no">
<meta name="color-scheme" content="$colorScheme">
<link rel="preconnect" href="https://fonts.googleapis.com">
<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
<link href="https://fonts.googleapis.com/css2?family=Geist:wght@400;500;600&family=Geist+Mono:wght@400;500&family=Newsreader:ital,opsz,wght@0,6..72,400;0,6..72,500;0,6..72,600;1,6..72,400..500&display=swap" rel="stylesheet">
<style>
$css
</style>
</head>
<body style="--veil-range: $veilRange">
${auraMarkup(artwork)}<div class="chrome-veil"></div><div class="grain"></div>
<div class="rscroll" style="padding-top: ${topContentPaddingPx + READER_MASTHEAD_TOP_GAP_PX}px">
$mastheadHtml$summaryBlockHtml<div class="article-body" id="article-body">$articleHtml</div>
</div>
<script nonce="$cspNonce">
$READER_BRIDGE_JS
$TRANSCRIPT_TAP_JS
var _highlights = $highlightJson;
applyHighlights(_highlights);
</script>
</body>
</html>
            """.trimIndent()
    }

    fun buildTypographyCss(
        preferences: ReaderPreferences,
        isDarkMode: Boolean = false,
    ): String = buildCss(preferences, isDarkMode)

    fun buildHighlightJson(highlights: List<HighlightData>): String {
        if (highlights.isEmpty()) return "[]"
        val entries =
            highlights
                .mapNotNull { h ->
                    val locator = h.locator ?: return@mapNotNull null
                    val startOffset = locator.startOffset ?: return@mapNotNull null
                    val endOffset = locator.endOffset ?: return@mapNotNull null
                    val tagsJson = h.tags.joinToString(",") { "\"${escapeJs(it)}\"" }
                    """{"id":"${escapeJs(h.id)}","color":"${escapeJs(h.color)}",""" +
                        """"start":$startOffset,"end":$endOffset,"tags":[$tagsJson]}"""
                }.joinToString(",")
        return "[$entries]"
    }

    // Canvas colors are driven by ReaderBackground; isDarkMode is kept for call-site compatibility.
    fun colorSchemeFor(
        preferences: ReaderPreferences,
        @Suppress("UNUSED_PARAMETER") isDarkMode: Boolean,
    ): String =
        when (preferences.background) {
            ReaderBackground.PAPER -> "only light"
            ReaderBackground.SEPIA -> "only light"
            ReaderBackground.SLATE -> "only dark"
            ReaderBackground.BLACK -> "only dark"
        }

    // A document with no drawing still gets the veils and grain; only the field is absent.
    private fun auraMarkup(artwork: LoadedReaderArtwork?): String =
        artwork?.let {
            """<div class="aura">${it.svg}</div>"""
        } ?: ""

    // Section order is load-bearing: later rules override earlier ones by cascade position.
    private fun buildCss(
        preferences: ReaderPreferences,
        isDarkMode: Boolean,
    ): String {
        val isDarkBg =
            preferences.background == ReaderBackground.SLATE ||
                preferences.background == ReaderBackground.BLACK
        val palette = if (isDarkBg) DARK_READER_PALETTE else LIGHT_READER_PALETTE
        val colors = backgroundColors(preferences.background)
        return listOf(
            buildReaderRootCss(preferences, colors, palette, colorSchemeFor(preferences, isDarkMode)),
            READER_RESET_CSS,
            READER_YOUTUBE_CSS,
            READER_PROSE_CSS,
            READER_MASTHEAD_CSS,
            buildReaderHighlightCss(palette, preferences.highlightStyle),
            buildReaderChromeCss(isDarkBg),
        ).joinToString("\n")
    }
}
