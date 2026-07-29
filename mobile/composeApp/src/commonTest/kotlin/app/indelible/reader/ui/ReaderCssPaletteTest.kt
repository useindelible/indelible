package app.indelible.reader.ui

import androidx.compose.ui.graphics.Color
import app.indelible.reader.model.ReaderBackground
import app.indelible.ui.theme.AccentDark
import app.indelible.ui.theme.AccentHoverLight
import app.indelible.ui.theme.AccentLight
import app.indelible.ui.theme.ReaderBgPaper
import kotlin.test.Test
import kotlin.test.assertEquals

/**
 * The reader's CSS palette is a set of hex string literals, not values derived from
 * [Color] at runtime: CSS needs strings, and several tokens are `rgba()` forms that a
 * Color-to-hex formatter would lose. Keeping the two in step is therefore a discipline
 * problem — so this test makes CI fail when they drift, instead of a reviewer having
 * to notice.
 */
class ReaderCssPaletteTest {
    private fun channel(value: Float): String {
        val scaled = (value * 255f).toInt().coerceIn(0, 255)
        return scaled.toString(16).uppercase().padStart(2, '0')
    }

    private fun Color.toCssHex(): String = "#${channel(red)}${channel(green)}${channel(blue)}"

    @Test
    fun lightReaderAccentMatchesTheAppAccent() {
        assertEquals(AccentLight.toCssHex(), LIGHT_READER_PALETTE.accent.uppercase())
    }

    @Test
    fun darkReaderAccentMatchesTheAppAccent() {
        assertEquals(AccentDark.toCssHex(), DARK_READER_PALETTE.accent.uppercase())
    }

    @Test
    fun paperCanvasMatchesTheReaderSurfaceToken() {
        assertEquals(
            ReaderBgPaper.toCssHex(),
            backgroundColors(ReaderBackground.PAPER).bg.uppercase(),
        )
    }

    @Test
    fun lightHighlightEdgeReusesTheAccentRatherThanAnInventedBlue() {
        // The blue highlight edge is the accent; a divergent hex here means someone
        // hand-picked a colour instead of reaching for the token.
        assertEquals(AccentLight.toCssHex(), LIGHT_READER_PALETTE.hlBEdge.uppercase())
    }

    @Test
    fun accentHoverIsNotSilentlyReusedAsTheAccent() {
        // Guards against pasting the hover value into the accent slot: they differ by
        // one step and the mistake is invisible on screen.
        assertEquals(AccentHoverLight.toCssHex(), "#0860CA")
        assertEquals("#0969DA", LIGHT_READER_PALETTE.accent.uppercase())
    }

    @Test
    fun everyBackgroundDeclaresAFullTriple() {
        ReaderBackground.entries.forEach { bg ->
            val c = backgroundColors(bg)
            listOf(c.bg, c.ink, c.body).forEach { value ->
                assertEquals(7, value.length, "$bg expects #RRGGBB, got $value")
            }
        }
    }
}
