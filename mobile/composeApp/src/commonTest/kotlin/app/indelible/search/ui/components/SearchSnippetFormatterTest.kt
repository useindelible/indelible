package app.indelible.search.ui.components

import androidx.compose.ui.graphics.Color
import kotlin.test.Test
import kotlin.test.assertEquals

class SearchSnippetFormatterTest {
    @Test
    fun parseSnippetHtml_stripsTagsDecodesEntitiesAndPreservesHighlightRange() {
        val highlightColor = Color.Blue
        val parsed = parseSnippetHtml("<p>alpha <mark>beta &amp; gamma</mark></p>", highlightColor)

        assertEquals("alpha beta & gamma", parsed.text)
        val highlight = parsed.spanStyles.single()
        assertEquals(6, highlight.start)
        assertEquals(18, highlight.end)
        assertEquals(highlightColor, highlight.item.background)
    }

    @Test
    fun parseSnippetHtml_stripsNestedTagsInsideHighlight() {
        val highlightColor = Color.Green
        val parsed = parseSnippetHtml("alpha <MARK><em>beta</em> &#x27;gamma&#39;</MARK>", highlightColor)

        assertEquals("alpha beta 'gamma'", parsed.text)
        val highlight = parsed.spanStyles.single()
        assertEquals(6, highlight.start)
        assertEquals(18, highlight.end)
        assertEquals(highlightColor, highlight.item.background)
    }

    @Test
    fun parseSnippetHtml_highlightsUntilEndWhenMarkIsUnclosed() {
        val highlightColor = Color.Red
        val parsed = parseSnippetHtml("<p>alpha <mark>beta&nbsp;<em>gamma</em></p>", highlightColor)

        assertEquals("alpha beta gamma", parsed.text)
        val highlight = parsed.spanStyles.single()
        assertEquals(6, highlight.start)
        assertEquals(16, highlight.end)
        assertEquals(highlightColor, highlight.item.background)
    }
}
