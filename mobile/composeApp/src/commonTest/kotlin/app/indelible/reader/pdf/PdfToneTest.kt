package app.indelible.reader.pdf

import app.indelible.reader.model.ReaderBackground
import kotlin.math.pow
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull

private fun ByteArray.channel(
    channel: Int,
    value: Int,
): Int = this[channel * 256 + value].toInt() and 0xFF

class PdfToneTest {
    @Test
    fun darkTableMapsWhiteToTheWebBackground() {
        val table = checkNotNull(toneTable(PdfTone.DARK))

        assertEquals(listOf(28, 28, 28), (0..2).map { table.channel(it, 255) })
        val pixels = intArrayOf(0xFFFFFFFF.toInt())
        applyToneArgb(pixels, table)
        assertEquals(0xFF1C1C1C.toInt(), pixels[0])
    }

    @Test
    fun darkTableMapsBlackToWhite() {
        val table = checkNotNull(toneTable(PdfTone.DARK))

        assertEquals(listOf(255, 255, 255), (0..2).map { table.channel(it, 0) })
    }

    @Test
    fun darkTableMatchesTheWebAt128() {
        val table = checkNotNull(toneTable(PdfTone.DARK))

        val expected = (255 - 227 * (128 / 255.0).pow(0.6)).toInt()
        assertEquals(expected, table.channel(0, 128))
    }

    @Test
    fun sepiaTableMatchesTheWeb() {
        val table = checkNotNull(toneTable(PdfTone.SEPIA))

        assertEquals(listOf(91, 70, 54), (0..2).map { table.channel(it, 0) })
        assertEquals(listOf(245, 237, 218), (0..2).map { table.channel(it, 255) })
    }

    @Test
    fun toneKeepsAlpha() {
        val table = checkNotNull(toneTable(PdfTone.DARK))
        val pixels = intArrayOf(0x80FFFFFF.toInt(), 0x00000000)

        applyToneArgb(pixels, table)

        assertEquals(0x801C1C1C.toInt(), pixels[0])
        assertEquals(0x00FFFFFF, pixels[1])
    }

    @Test
    fun backgroundsPickTheirTone() {
        assertEquals(PdfTone.NONE, toneFor(ReaderBackground.PAPER))
        assertEquals(PdfTone.SEPIA, toneFor(ReaderBackground.SEPIA))
        assertEquals(PdfTone.DARK, toneFor(ReaderBackground.SLATE))
        assertEquals(PdfTone.DARK, toneFor(ReaderBackground.BLACK))
        assertNull(toneTable(PdfTone.NONE))
    }
}
