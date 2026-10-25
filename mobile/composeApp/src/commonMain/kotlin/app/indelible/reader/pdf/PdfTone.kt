package app.indelible.reader.pdf

import androidx.compose.ui.graphics.Color
import app.indelible.reader.model.ReaderBackground
import kotlin.math.pow

private const val LEVELS = 256
private const val CHANNELS = 3
private const val MAX = 255
private const val DARK_PAGE = 28
private const val DARK_CURVE = 0.6
private const val SEPIA_INK_R = 91
private const val SEPIA_INK_G = 70
private const val SEPIA_INK_B = 54
private const val SEPIA_PAGE_R = 245
private const val SEPIA_PAGE_G = 237
private const val SEPIA_PAGE_B = 218
private const val RED_SHIFT = 16
private const val GREEN_SHIFT = 8
private const val CHANNEL_MASK = 0xFF
private const val ALPHA_MASK = 0xFF000000.toInt()
private val SEPIA_INK = intArrayOf(SEPIA_INK_R, SEPIA_INK_G, SEPIA_INK_B)
private val SEPIA_PAGE = intArrayOf(SEPIA_PAGE_R, SEPIA_PAGE_G, SEPIA_PAGE_B)

/** Three 256-entry channel tables (R, G, B) matching the web reader's lookups; null for [PdfTone.NONE]. */
fun toneTable(tone: PdfTone): ByteArray? {
    val level: (channel: Int, value: Int) -> Int =
        when (tone) {
            PdfTone.NONE -> return null
            PdfTone.DARK -> { _, value -> (MAX - (MAX - DARK_PAGE) * (value / MAX.toDouble()).pow(DARK_CURVE)).toInt() }
            PdfTone.SEPIA -> { channel, value ->
                (SEPIA_INK[channel] + (SEPIA_PAGE[channel] - SEPIA_INK[channel]) * (value / MAX.toDouble())).toInt()
            }
        }
    return ByteArray(CHANNELS * LEVELS) { index -> level(index / LEVELS, index % LEVELS).toByte() }
}

/** Maps every pixel's R, G and B through [table]; alpha is kept. */
fun applyToneArgb(
    pixels: IntArray,
    table: ByteArray,
) {
    for (i in pixels.indices) {
        val argb = pixels[i]
        val r = table[(argb shr RED_SHIFT) and CHANNEL_MASK].toInt() and CHANNEL_MASK
        val g = table[LEVELS + ((argb shr GREEN_SHIFT) and CHANNEL_MASK)].toInt() and CHANNEL_MASK
        val b = table[2 * LEVELS + (argb and CHANNEL_MASK)].toInt() and CHANNEL_MASK
        pixels[i] = (argb and ALPHA_MASK) or (r shl RED_SHIFT) or (g shl GREEN_SHIFT) or b
    }
}

fun toneFor(background: ReaderBackground): PdfTone =
    when (background) {
        ReaderBackground.PAPER -> PdfTone.NONE
        ReaderBackground.SEPIA -> PdfTone.SEPIA
        ReaderBackground.SLATE, ReaderBackground.BLACK -> PdfTone.DARK
    }

/** What white paper looks like under [tone]: the placeholder behind a page until its bitmap arrives. */
fun toneBackground(tone: PdfTone): Color {
    val table = toneTable(tone) ?: return Color.White
    val white = LEVELS - 1
    return Color(
        red = table[white].toInt() and CHANNEL_MASK,
        green = table[LEVELS + white].toInt() and CHANNEL_MASK,
        blue = table[2 * LEVELS + white].toInt() and CHANNEL_MASK,
    )
}
