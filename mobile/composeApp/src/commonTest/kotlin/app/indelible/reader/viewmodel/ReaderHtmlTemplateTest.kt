package app.indelible.reader.viewmodel

import app.indelible.reader.model.HighlightColor
import app.indelible.reader.model.HighlightData
import app.indelible.reader.model.HighlightLocator
import app.indelible.reader.model.ReaderBackground
import app.indelible.reader.model.ReaderPreferences
import app.indelible.reader.model.TextAlign
import app.indelible.reader.model.Typeface
import app.indelible.reader.ui.ReaderHtmlLocalization
import app.indelible.reader.ui.ReaderHtmlTemplate
import kotlinx.datetime.Instant
import kotlin.test.Test
import kotlin.test.assertContains
import kotlin.test.assertEquals
import kotlin.test.assertFalse

class ReaderHtmlTemplateTest {
    @Test
    fun build_contains_article_body() {
        val html =
            ReaderHtmlTemplate.build(
                articleHtml = "<p>Test content</p>",
                preferences = ReaderPreferences(),
                highlights = emptyList(),
            )
        assertContains(html, "<p>Test content</p>")
        assertContains(html, "article-body")
    }

    @Test
    fun build_uses_preformatted_localized_metadata() {
        val html =
            ReaderHtmlTemplate.build(
                articleHtml = "<p>x</p>",
                preferences = ReaderPreferences(),
                highlights = emptyList(),
                articleTitle = "Un article",
                localization =
                    ReaderHtmlLocalization(
                        publishedDate = "5 mars 2024",
                        readingTime = "1 minute",
                        summaryLabel = "Résumé Mila",
                        askFollowUpLabel = "Poser une question",
                    ),
            )
        assertContains(html, "5 mars 2024")
        assertContains(html, "1 minute")
        assertFalse(html.contains("March 5, 2024"))
    }

    @Test
    fun build_includes_font_family_for_serif() {
        val html =
            ReaderHtmlTemplate.build(
                articleHtml = "<p>Test</p>",
                preferences = ReaderPreferences(typeface = Typeface.SERIF),
                highlights = emptyList(),
            )
        assertContains(html, "Georgia")
    }

    @Test
    fun serif_template_contains_newsreader_and_pullquote() {
        val css =
            ReaderHtmlTemplate.buildTypographyCss(
                ReaderPreferences(typeface = Typeface.SERIF, background = ReaderBackground.PAPER),
                isDarkMode = false,
            )
        assertContains(css, "Newsreader")
        assertContains(css, ".pullquote")
        assertContains(css, "#FBFAF6")
    }

    @Test
    fun build_includes_font_family_for_mono() {
        val html =
            ReaderHtmlTemplate.build(
                articleHtml = "<p>Test</p>",
                preferences = ReaderPreferences(typeface = Typeface.MONO),
                highlights = emptyList(),
            )
        assertContains(html, "SF Mono")
    }

    @Test
    fun build_includes_font_size() {
        val html =
            ReaderHtmlTemplate.build(
                articleHtml = "<p>Test</p>",
                preferences = ReaderPreferences(fontSize = 22),
                highlights = emptyList(),
            )
        assertContains(html, "22px")
    }

    @Test
    fun build_includes_line_height() {
        val html =
            ReaderHtmlTemplate.build(
                articleHtml = "<p>Test</p>",
                preferences = ReaderPreferences(lineHeight = 1.9f),
                highlights = emptyList(),
            )
        assertContains(html, "1.9")
    }

    @Test
    fun build_includes_justified_alignment() {
        val html =
            ReaderHtmlTemplate.build(
                articleHtml = "<p>Test</p>",
                preferences = ReaderPreferences(textAlign = TextAlign.JUSTIFIED),
                highlights = emptyList(),
            )
        assertContains(html, "justify")
    }

    @Test
    fun dark_background_uses_dark_colors() {
        val html =
            ReaderHtmlTemplate.build(
                articleHtml = "<p>Test</p>",
                preferences = ReaderPreferences(background = ReaderBackground.BLACK),
                highlights = emptyList(),
            )
        assertContains(html, "#0D1117")
        assertContains(html, "#E9EEF4")
    }

    @Test
    fun sepia_background_uses_sepia_colors() {
        val html =
            ReaderHtmlTemplate.build(
                articleHtml = "<p>Test</p>",
                preferences = ReaderPreferences(background = ReaderBackground.SEPIA),
                highlights = emptyList(),
            )
        assertContains(html, "#F4ECD8")
        assertContains(html, "#5B4636")
    }

    @Test
    fun body_background_is_transparent_for_aura() {
        val html =
            ReaderHtmlTemplate.build(
                articleHtml = "<p>Test</p>",
                preferences = ReaderPreferences(),
                highlights = emptyList(),
            )
        assertContains(html, "background: transparent")
        assertFalse(html.contains("prefers-color-scheme"))
    }

    @Test
    fun top_content_padding_offsets_body_for_native_chrome() {
        val html =
            ReaderHtmlTemplate.build(
                articleHtml = "<p>Test</p>",
                preferences = ReaderPreferences(),
                highlights = emptyList(),
                topContentPaddingPx = 80,
            )
        // 80px native chrome + 26px base masthead gap.
        assertContains(html, "padding-top: 106px")
    }

    @Test
    fun highlights_use_inset_edge_rule() {
        val html =
            ReaderHtmlTemplate.build(
                articleHtml = "<p>Test</p>",
                preferences = ReaderPreferences(),
                highlights = emptyList(),
            )
        assertContains(html, "inset 0 -2px 0")
    }

    @Test
    fun summary_block_present_when_summary_supplied() {
        val html =
            ReaderHtmlTemplate.build(
                articleHtml = "<p>Test</p>",
                preferences = ReaderPreferences(),
                highlights = emptyList(),
                summaryHtml = "A concise overview of the article.",
                summaryPoints = listOf("First key point", "Second key point"),
            )
        assertContains(html, "reader-summary")
        assertContains(html, "A concise overview of the article.")
        assertContains(html, "First key point")
        assertContains(html, "onSummaryAction")
    }

    @Test
    fun summary_block_absent_when_no_summary() {
        val html =
            ReaderHtmlTemplate.build(
                articleHtml = "<p>Test</p>",
                preferences = ReaderPreferences(),
                highlights = emptyList(),
            )
        assertFalse(html.contains("reader-summary"))
    }

    @Test
    fun set_speaking_helper_present() {
        val html =
            ReaderHtmlTemplate.build(
                articleHtml = "<p>Test</p>",
                preferences = ReaderPreferences(),
                highlights = emptyList(),
            )
        assertContains(html, "setSpeaking")
        assertContains(html, ".say")
    }

    @Test
    fun build_highlight_json_empty() {
        val json = ReaderHtmlTemplate.buildHighlightJson(emptyList())
        assertEquals("[]", json)
    }

    @Test
    fun build_highlight_json_with_entries() {
        val highlights =
            listOf(
                HighlightData(
                    id = "hlt_1",
                    documentId = "doc_1",
                    color = "yellow",
                    textContent = "text",
                    locator = HighlightLocator(type = "html", startOffset = 10, endOffset = 20),
                    tags = emptyList(),
                    createdAt = Instant.parse("2024-01-01T00:00:00Z"),
                    updatedAt = Instant.parse("2024-01-01T00:00:00Z"),
                ),
            )
        val json = ReaderHtmlTemplate.buildHighlightJson(highlights)
        assertContains(json, "hlt_1")
        assertContains(json, "\"color\":\"yellow\"")
        assertContains(json, "\"start\":10")
        assertContains(json, "\"end\":20")
    }

    @Test
    fun build_includes_highlight_css_classes() {
        val html =
            ReaderHtmlTemplate.build(
                articleHtml = "<p>Test</p>",
                preferences = ReaderPreferences(),
                highlights = emptyList(),
            )
        // The runtime applies `mark.className = 'hl-' + color`, where color is the
        // lowercase apiValue, so every CSS rule must use that exact lowercase class.
        HighlightColor.entries.forEach { color ->
            assertContains(html, "hl-${color.apiValue}")
        }
    }

    @Test
    fun build_includes_content_security_policy_and_script_nonce() {
        val html =
            ReaderHtmlTemplate.build(
                articleHtml = "<p>Test</p>",
                preferences = ReaderPreferences(),
                highlights = emptyList(),
            )
        assertContains(html, "Content-Security-Policy")
        assertContains(html, "script-src 'nonce-")
        assertContains(html, "connect-src 'none'")
        // The reader's own bridge script must carry the nonce so the strict CSP still runs it.
        assertContains(html, "<script nonce=\"")
    }

    @Test
    fun build_includes_bridge_js() {
        val html =
            ReaderHtmlTemplate.build(
                articleHtml = "<p>Test</p>",
                preferences = ReaderPreferences(),
                highlights = emptyList(),
            )
        assertContains(html, "NativeBridge")
        assertContains(html, "scrollToPercent")
        assertContains(html, "updateTypography")
        assertContains(html, "applyHighlights")
    }

    @Test
    fun selection_end_offset_shares_node_map_basis() {
        val html =
            ReaderHtmlTemplate.build(
                articleHtml = "<p>Test</p>",
                preferences = ReaderPreferences(),
                highlights = emptyList(),
            )
        // applyHighlights measures offsets in raw text-node length (node.length).
        // The selection end offset must use that same basis: a Range to the
        // selection's end boundary. Deriving it from Selection.toString().length
        // (whitespace-collapsed rendered text) undercounts across inter-tag
        // whitespace and truncates the highlight end.
        assertContains(html, "range.endContainer")
        assertContains(html, "range.endOffset")
        assertFalse(html.contains("startOffset + text.length"))
    }

    @Test
    fun build_typography_css_returns_valid_css() {
        val css =
            ReaderHtmlTemplate.buildTypographyCss(
                ReaderPreferences(fontSize = 20, typeface = Typeface.SANS),
            )
        assertContains(css, "--font-size: 20px")
        assertContains(css, "Geist")
    }
}
