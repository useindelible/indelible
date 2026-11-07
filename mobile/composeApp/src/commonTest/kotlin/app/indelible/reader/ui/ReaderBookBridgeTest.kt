package app.indelible.reader.ui

import app.indelible.reader.model.ReaderPreferences
import kotlin.test.Test
import kotlin.test.assertContains
import kotlin.test.assertFalse

class ReaderBookBridgeTest {
    private fun page(
        html: String = "<p>x</p>",
        hold: Boolean = false,
        topInset: Int = 0,
    ) = ReaderHtmlTemplate.build(
        articleHtml = html,
        preferences = ReaderPreferences(),
        highlights = emptyList(),
        holdUntilPositioned = hold,
        topContentPaddingPx = topInset,
    )

    @Test
    fun heldTemplateHidesTheScrollerUntilPositioned() {
        val html = page(hold = true)

        assertContains(html, """<div class="rscroll hold"""")
        assertContains(html, ".rscroll.hold { visibility: hidden; }")
    }

    @Test
    fun unheldTemplateHasNoHoldClass() {
        assertFalse(page().contains("rscroll hold"))
    }

    @Test
    fun sampleCarriesPercentOffsetAndEndTogether() {
        val html = page()

        assertContains(html, "NativeBridge.onScroll(s.percent, s.y, s.offset, s.atEnd)")
        assertContains(html, "offset: complete ? firstVisibleOffset() : -1")
        assertContains(html, "scroller.scrollTop + scroller.clientHeight >= scroller.scrollHeight - 2")
    }

    @Test
    fun theSavedLineIsReadAndRestoredBelowTheTopChrome() {
        val html = page(topInset = 80)

        assertContains(html, "data-reading-line=\"80\"")
        assertContains(html, "var top = scroller.getBoundingClientRect().top + readingLine + 1;")
        assertContains(html, "box.top - scroller.getBoundingClientRect().top - readingLine")
    }

    @Test
    fun highlightLabelsKeepTheReadingLineWhereItWas() {
        val html = page()

        assertContains(html, "var applyHighlights = window.applyHighlights;")
        assertContains(html, "keepReadingLine(function() { applyHighlights(highlights); });")
        assertContains(html, "scroller.scrollTop += moved.top - before.top;")
    }

    @Test
    fun captureSampleIsDefined() {
        assertContains(page(), "window.captureSample = function")
    }

    @Test
    fun restoreWaitsForFontsAndImagesWithABound() {
        val html = page()

        assertContains(html, "window.scrollToCharOffset = function")
        assertContains(html, "document.fonts.ready")
        assertContains(html, "img.decode()")
        assertContains(html, "setTimeout(finish, 1200)")
        assertContains(html, "setTimeout(reveal, 1500)")
    }

    @Test
    fun anchorSearchFallsBackToName() {
        assertContains(page(), """root.querySelector('[name="' + CSS.escape(id) + '"]')""")
    }

    @Test
    fun bridgeInterceptsOnlyInBookLinks() {
        val html = page()

        assertContains(html, "closest('#article-body a[href]')")
        assertContains(html, "/^(https?|mailto):/i.test(href)")
        assertContains(html, "NativeBridge.onBookLink(href)")
    }

    @Test
    fun chapterHtmlIsEmbeddedVerbatim() {
        val chapter = """<h2 id="c3">Three &amp; four</h2><p><a name="fn4"></a>Note <a href="n.xhtml#fn4">4</a></p>"""

        assertContains(page(chapter), """<div class="article-body" id="article-body">$chapter</div>""")
    }

    @Test
    fun restoreSkipsWhitespaceWithoutABox() {
        val html = page()

        assertContains(html, "if (seen + node.length > offset)")
        assertContains(html, "while (i >= 0 && !/\\S/.test(text.charAt(i))) i--;")
        assertContains(html, "if (/\\S/.test(node.data)) last = node;")
    }

    @Test
    fun offsetsLeaveOutTagLabels() {
        val html = page()

        assertContains(html, "if (label.contains(container)) part.setEnd(container, offset);")
        assertContains(html, "window.readerTextOffset(range.startContainer, range.startOffset)")
        assertContains(html, "window.readerTextOffset(caret.startContainer, caret.startOffset)")
    }

    @Test
    fun missedCaretFallsInsideTheCrossingNode() {
        assertContains(page(), "return count + Math.round(node.length * above);")
    }

    @Test
    fun linkInsideAHighlightIsNotAHighlightTap() {
        assertContains(page(), "if (!mark || e.target.closest('a[href]')) return;")
    }
}
