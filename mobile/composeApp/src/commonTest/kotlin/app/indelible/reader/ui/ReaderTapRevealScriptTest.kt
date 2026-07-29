package app.indelible.reader.ui

import kotlin.test.Test
import kotlin.test.assertContains
import kotlin.test.assertFalse
import kotlin.test.assertTrue

class ReaderTapRevealScriptTest {
    @Test
    fun bridgeExposesTheTapChannel() {
        assertContains(READER_BRIDGE_JS, "NativeBridge.onReaderTap()")
    }

    @Test
    fun revealListensOnClickAndNeverOnTouchStart() {
        // 'click' is only synthesised after a tap that was not a scroll, a drag, or a
        // long-press selection. touchstart fires at the beginning of every swipe, which
        // would flash the chrome back on each time the reader is scrolled.
        assertContains(READER_BRIDGE_JS, "document.addEventListener('click'")
        assertFalse(
            READER_BRIDGE_JS.contains("addEventListener('touchstart'"),
            "touchstart would reveal on every swipe",
        )
        assertFalse(
            READER_BRIDGE_JS.contains("addEventListener('touchend'"),
            "touchend fires before a tap is disambiguated from a drag",
        )
    }

    @Test
    fun revealYieldsToControlsThatRaiseTheirOwnSurface() {
        val guard = READER_BRIDGE_JS.substringAfter("onReaderTap").let { READER_BRIDGE_JS }
        assertContains(guard, "mark[data-highlight-id]")
        assertContains(guard, ".t-seg")
        assertTrue(guard.contains("isCollapsed"), "an active selection must not count as a tap")
    }

    @Test
    fun scrollReportingIsCoalescedToOneCallPerFrame() {
        // Every scroll tick re-arms a native retire timer, and a WebView emits many per frame.
        assertContains(READER_BRIDGE_JS, "requestAnimationFrame(paintScroll)")
        assertContains(READER_BRIDGE_JS, "{ passive: true }")
    }

    @Test
    fun scrollingDrivesTheChoreographyVariables() {
        assertContains(READER_BRIDGE_JS, "setProperty('--y'")
        assertContains(READER_BRIDGE_JS, "setProperty('--p'")
        assertContains(READER_BRIDGE_JS, "setProperty('--aura-travel'")
    }

    @Test
    fun frameHeightIsDrivenFromInnerHeightNotFromCss() {
        // Android WebView resolves 100%/100vh/100dvh to 0 for this document, which
        // collapses the frame: no article body renders and nothing scrolls. The script
        // has to make the frame definite from the height the WebView actually reports.
        assertContains(READER_BRIDGE_JS, "window.innerHeight + 'px'")
        assertContains(READER_BRIDGE_JS, "document.documentElement.style.height")
        assertContains(READER_BRIDGE_JS, "document.body.style.height")
    }

    @Test
    fun frameHeightIsRecomputedWhenTheViewportChanges() {
        assertContains(READER_BRIDGE_JS, "addEventListener('resize', sizeFrame)")
        assertContains(READER_BRIDGE_JS, "addEventListener('orientationchange', sizeFrame)")
        assertContains(READER_BRIDGE_JS, "visualViewport")
    }

    @Test
    fun auraTravelIsMeasuredWithAnApiThatWorksOnSvg() {
        // The drawing is an <svg>, i.e. an SVGElement, which has no offsetHeight.
        // Reading offsetHeight returns undefined, leaves --aura-travel unset, and pins
        // the drawing over the article instead of letting it travel out of frame.
        assertContains(READER_BRIDGE_JS, "getBoundingClientRect().height")
        assertFalse(
            READER_BRIDGE_JS.contains("art.offsetHeight"),
            "offsetHeight is undefined on an SVG element",
        )
    }

    @Test
    fun mastheadControlsAreBoundInScriptNotByInlineAttributes() {
        // script-src carries a nonce and no 'unsafe-inline', so an onclick attribute is
        // never compiled into a handler and the control is silently inert. The summary
        // disclosure and its follow-up button have to be bound from the nonced script.
        assertContains(READER_BRIDGE_JS, "window.toggleReaderSummary()")
        assertContains(READER_BRIDGE_JS, "window.readerSummaryAction('ask')")
        assertContains(READER_BRIDGE_JS, ".sum-toggle")
    }

    @Test
    fun scrollTargetsTheInnerScrollerNotTheDocument() {
        assertContains(READER_BRIDGE_JS, "document.querySelector('.rscroll')")
        assertContains(READER_BRIDGE_JS, "scroller.scrollTop")
        assertFalse(READER_BRIDGE_JS.contains("window.scrollTo("), "body scrolling was replaced")
    }
}
