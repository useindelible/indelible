@file:OptIn(ExperimentalForeignApi::class)

package app.indelible.reader.pdf

import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.graphics.toComposeImageBitmap
import kotlinx.cinterop.ExperimentalForeignApi
import kotlinx.cinterop.addressOf
import kotlinx.cinterop.autoreleasepool
import kotlinx.cinterop.usePinned
import org.jetbrains.skia.ColorAlphaType
import org.jetbrains.skia.ColorType
import org.jetbrains.skia.Image
import org.jetbrains.skia.ImageInfo
import platform.CoreGraphics.CGBitmapContextCreate
import platform.CoreGraphics.CGColorSpaceCreateDeviceRGB
import platform.CoreGraphics.CGColorSpaceRelease
import platform.CoreGraphics.CGContextFillRect
import platform.CoreGraphics.CGContextRelease
import platform.CoreGraphics.CGContextScaleCTM
import platform.CoreGraphics.CGContextSetRGBFillColor
import platform.CoreGraphics.CGImageAlphaInfo
import platform.CoreGraphics.CGRectMake
import platform.CoreGraphics.kCGBitmapByteOrder32Little
import platform.PDFKit.PDFPage
import platform.PDFKit.kPDFDisplayBoxCropBox

private const val BYTES_PER_PIXEL = 4
private const val BITS_PER_COMPONENT = 8uL
private const val LEVELS = 256
private const val BYTE_MASK = 0xFF

/** Draws [page] into a BGRA buffer at the bounded size for [widthPx], applies [tone], and wraps it as a bitmap. */
internal fun renderPage(
    page: PDFPage,
    size: PdfPageSize,
    widthPx: Int,
    tone: PdfTone,
): ImageBitmap {
    val (w, h) = renderSizeFor(size, widthPx)
    val bytes = ByteArray(bufferBytes(w, h))
    bytes.usePinned { pinned ->
        val space = CGColorSpaceCreateDeviceRGB()
        val info = CGImageAlphaInfo.kCGImageAlphaPremultipliedFirst.value or kCGBitmapByteOrder32Little
        val rowBytes = (w * BYTES_PER_PIXEL).toULong()
        val data = pinned.addressOf(0)
        val context = CGBitmapContextCreate(data, w.toULong(), h.toULong(), BITS_PER_COMPONENT, rowBytes, space, info)
        CGContextSetRGBFillColor(context, 1.0, 1.0, 1.0, 1.0)
        CGContextFillRect(context, CGRectMake(0.0, 0.0, w.toDouble(), h.toDouble()))
        // drawWithBox applies the crop box and /Rotate itself; only the scale is set here.
        CGContextScaleCTM(context, w / size.width, h / size.height)
        autoreleasepool { page.drawWithBox(kPDFDisplayBoxCropBox, toContext = context) }
        CGContextRelease(context)
        CGColorSpaceRelease(space)
    }
    toneTable(tone)?.let { applyToneBgra(bytes, it) }
    val info = ImageInfo(w, h, ColorType.BGRA_8888, ColorAlphaType.PREMUL)
    return Image.makeRaster(info, bytes, w * BYTES_PER_PIXEL).toComposeImageBitmap()
}

private fun applyToneBgra(
    bytes: ByteArray,
    table: ByteArray,
) {
    var i = 0
    while (i < bytes.size) {
        bytes[i] = table[2 * LEVELS + (bytes[i].toInt() and BYTE_MASK)]
        bytes[i + 1] = table[LEVELS + (bytes[i + 1].toInt() and BYTE_MASK)]
        bytes[i + 2] = table[bytes[i + 2].toInt() and BYTE_MASK]
        i += BYTES_PER_PIXEL
    }
}
