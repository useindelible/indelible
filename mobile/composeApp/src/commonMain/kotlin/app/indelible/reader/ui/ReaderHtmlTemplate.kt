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
    private const val NONCE_BYTES = 16
    private const val BYTE_MASK = 0xFF
    private const val HEX_RADIX = 16

    // Per-render CSP nonce: the reader's own inline bridge script carries it so the strict
    // script-src can run it while blocking any injected inline script or event-handler attribute.
    private fun randomNonce(): String =
        Random.nextBytes(NONCE_BYTES).joinToString("") {
            (it.toInt() and BYTE_MASK).toString(HEX_RADIX).padStart(2, '0')
        }

    fun build(
        articleHtml: String,
        preferences: ReaderPreferences,
        highlights: List<HighlightData>,
        options: ReaderPageOptions = ReaderPageOptions(),
    ): String {
        val css = buildCss(preferences)
        val scrollerClass = if (options.holdUntilPositioned) "rscroll hold" else "rscroll"
        val assets = options.assets
        val veilRange = assets.artwork?.artwork?.veilRange ?: DEFAULT_VEIL_RANGE
        val cspNonce = randomNonce()
        val colorScheme = colorSchemeFor(preferences)
        val highlightJson = buildHighlightJson(highlights)
        val topContentPaddingPx = options.topContentPaddingPx
        val summary = options.summary
        val localization = options.localization
        val mastheadHtml =
            buildMasthead(
                ReaderMastheadData(
                    title = options.article.title,
                    author = options.article.author,
                    domain = options.article.domain,
                    hasSummary = summary.html != null,
                ),
                localization = localization,
            )
        val summaryBlockHtml = buildSummaryBlock(summary.html, summary.points, localization.askFollowUpLabel)
        return """
<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<meta http-equiv="Content-Security-Policy" content="default-src 'none'; script-src 'nonce-$cspNonce'; style-src 'unsafe-inline' https:; font-src data:; img-src https: http: data:; media-src https: http: data:; frame-src https://www.youtube.com https://www.youtube-nocookie.com; connect-src 'none'; base-uri 'none'; form-action 'none'">
<meta name="viewport" content="width=device-width, initial-scale=1, maximum-scale=1, user-scalable=no">
<meta name="color-scheme" content="$colorScheme">
<style>
${assets.fontFaceCss}
$css
</style>
</head>
<body style="--veil-range: $veilRange">
${auraMarkup(assets.artwork)}<div class="chrome-veil"></div><div class="grain"></div>
<div class="$scrollerClass" data-reading-line="$topContentPaddingPx" style="padding-top: ${topContentPaddingPx + READER_MASTHEAD_TOP_GAP_PX}px">
$mastheadHtml$summaryBlockHtml<div class="article-body" id="article-body">$articleHtml</div>
</div>
<script nonce="$cspNonce">
$READER_BRIDGE_JS
$TRANSCRIPT_TAP_JS
$READER_BOOK_JS
var _highlights = $highlightJson;
applyHighlights(_highlights);
</script>
</body>
</html>
            """.trimIndent()
    }

    fun buildTypographyCss(preferences: ReaderPreferences): String = buildCss(preferences)

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

    fun colorSchemeFor(preferences: ReaderPreferences): String =
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
    private fun buildCss(preferences: ReaderPreferences): String {
        val isDarkBg =
            preferences.background == ReaderBackground.SLATE ||
                preferences.background == ReaderBackground.BLACK
        val palette = if (isDarkBg) DARK_READER_PALETTE else LIGHT_READER_PALETTE
        val colors = backgroundColors(preferences.background)
        return listOf(
            buildReaderRootCss(preferences, colors, palette, colorSchemeFor(preferences)),
            READER_RESET_CSS,
            READER_YOUTUBE_CSS,
            READER_PROSE_CSS,
            READER_MASTHEAD_CSS,
            buildReaderHighlightCss(palette, preferences.highlightStyle),
            buildReaderChromeCss(isDarkBg),
            READER_BOOK_CSS,
        ).joinToString("\n")
    }
}
