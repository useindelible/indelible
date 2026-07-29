package app.indelible.reader.ui

import app.indelible.reader.model.ReaderArtwork
import app.indelible.reader.model.ReaderBackground
import app.indelible.reader.model.ReaderPreferences
import kotlin.test.Test
import kotlin.test.assertContains
import kotlin.test.assertFalse
import kotlin.test.assertTrue

private const val FAKE_SVG = """<svg viewBox="0 0 2172 724"><rect width="10" height="10"/></svg>"""

private fun html(
    background: ReaderBackground = ReaderBackground.PAPER,
    artwork: LoadedReaderArtwork? = LoadedReaderArtwork(ReaderArtwork.MISTY_RIDGES, FAKE_SVG),
): String =
    ReaderHtmlTemplate.build(
        articleHtml = "<p>Test</p>",
        preferences = ReaderPreferences(background = background),
        highlights = emptyList(),
        artwork = artwork,
    )

class ReaderChromeCssTest {
    @Test
    fun drawing_is_inlined_inside_the_aura_layer() {
        val out = html()
        assertContains(out, """<div class="aura">""")
        assertContains(out, FAKE_SVG)
    }

    @Test
    fun field_travels_its_own_measured_height_not_a_hardcoded_distance() {
        // The cap is the drawing's rendered height, written by script, because the
        // drawn height follows the frame width and so differs per device.
        assertContains(html(), "var(--aura-travel")
        assertContains(html(), "calc(var(--y) * 0.5px)")
    }

    @Test
    fun scrim_is_driven_by_scroll_position() {
        val out = html()
        assertContains(out, "calc(1 - clamp(0, calc(var(--y) / var(--veil-range)), 1))")
    }

    @Test
    fun veil_range_is_published_per_drawing() {
        assertContains(
            html(artwork = LoadedReaderArtwork(ReaderArtwork.PINE_RIDGES, FAKE_SVG)),
            "--veil-range: 130",
        )
        assertContains(
            html(artwork = LoadedReaderArtwork(ReaderArtwork.MISTY_RIDGES, FAKE_SVG)),
            "--veil-range: 60",
        )
    }

    @Test
    fun tint_is_never_emitted() {
        // The prototype's tint slider is deliberately not shipped: drawings always
        // render in their authored palette.
        ReaderBackground.entries.forEach { bg ->
            assertFalse(html(bg).contains("hue-rotate"), "hue-rotate leaked on $bg")
            assertFalse(html(bg).contains("--art-rot"), "--art-rot leaked on $bg")
        }
    }

    @Test
    fun light_papers_emit_no_filter_and_dark_papers_dim_the_drawing() {
        // On paper the authored filter is a visual no-op but still forces a composite
        // pass over a subtree carrying nine Gaussian blurs, so it is omitted entirely.
        assertContains(html(ReaderBackground.PAPER), "filter: none")
        assertContains(html(ReaderBackground.SEPIA), "filter: none")
        assertContains(html(ReaderBackground.SLATE), "saturate(1.35) brightness(0.52) contrast(1.12)")
        assertContains(html(ReaderBackground.BLACK), "saturate(1.35) brightness(0.52) contrast(1.12)")
    }

    @Test
    fun document_uses_an_inner_scroller_so_the_field_and_veils_can_stand_still() {
        val out = html()
        assertContains(out, """<div class="rscroll"""")
        assertContains(out, "overflow-y: auto")
        // Ordering that keeps the scrim off the prose: art(0) < scrim(1) < prose(10).
        assertContains(out, "z-index: 10")
        assertContains(out, "z-index: 1;")
    }

    @Test
    fun a_document_without_a_drawing_still_gets_the_scrim() {
        val out = html(artwork = null)
        assertFalse(out.contains("""<div class="aura">"""))
        assertContains(out, """<div class="chrome-veil">""")
        assertTrue(out.contains("--veil-range: 60"), "fallback veil range")
    }

    @Test
    fun scrim_retires_with_the_controls_it_backs() {
        // The scrim exists for the floating controls. Left up on its own it reads as a
        // band of page colour across the top and the article visibly stops short of the
        // edge instead of running under the camera.
        val out = html()
        assertContains(out, "body.immersive .chrome-veil")
        assertContains(out, "translateY(-100%)")
        // The status bar never returns while the reader is open, so the strip that
        // existed only to keep prose off its glyphs is gone entirely.
        assertFalse(out.contains("text-veil"), "text-veil should no longer be emitted")
    }

    @Test
    fun immersive_state_can_be_pushed_into_the_document() {
        assertContains(READER_BRIDGE_JS, "window.setReaderImmersive")
        assertContains(READER_BRIDGE_JS, "classList.toggle('immersive'")
    }

    @Test
    fun markup_emits_no_inline_event_handlers() {
        // Anything relying on one would be dead under the document's nonce-only script-src.
        assertFalse(html().contains("onclick="), "inline handlers never run under this CSP")
    }

    @Test
    fun grain_is_present_on_every_paper() {
        ReaderBackground.entries.forEach { bg ->
            assertContains(html(bg), """<div class="grain">""")
            assertContains(html(bg), "mix-blend-mode:")
        }
    }
}
