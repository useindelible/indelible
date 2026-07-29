package app.indelible.reader.ui

import app.indelible.reader.model.ReaderPreferences
import kotlin.test.Test
import kotlin.test.assertFalse
import kotlin.test.assertTrue

/**
 * Video documents render their masthead natively, so the template must omit its own
 * masthead for them while still emitting the summary handle the reader taps to expand.
 */
class ReaderHtmlTemplateMastheadTest {
    private fun build(
        title: String,
        author: String?,
        domain: String?,
        summary: String?,
    ) = ReaderHtmlTemplate.build(
        articleHtml = "<p>body</p>",
        preferences = ReaderPreferences(),
        highlights = emptyList(),
        articleTitle = title,
        articleAuthor = author,
        articleDomain = domain,
        summaryHtml = summary,
    )

    @Test
    fun articleKeepsItsMastheadAndTitle() {
        val html = build("The End of the Beginning", "Ben Thompson", "stratechery.com", "A summary.")

        assertTrue(html.contains("<header class=\"masthead\">"), "article should render the template masthead")
        assertTrue(html.contains("The End of the Beginning"), "article title should be present")
        assertTrue(html.contains("reader-summary-handle"), "summary handle should be present")
    }

    @Test
    fun videoOmitsMastheadButKeepsSummaryHandle() {
        val html = build(title = "", author = null, domain = null, summary = "A summary.")

        assertFalse(html.contains("<header class=\"masthead\">"), "video should not render a second masthead")
        assertTrue(html.contains("reader-summary-handle"), "summary handle must survive")
        assertTrue(html.contains("mh-rule"), "the rule the disclosure sits under must survive")
    }

    @Test
    fun videoWithoutSummaryRendersNoMastheadBlock() {
        val html = build(title = "", author = null, domain = null, summary = null)

        assertFalse(html.contains("<header class=\"masthead\">"), "video should not render a masthead")
        assertFalse(html.contains("reader-summary-handle"), "no summary means no handle")
    }
}
