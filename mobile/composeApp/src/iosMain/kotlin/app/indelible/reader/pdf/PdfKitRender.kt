@file:OptIn(ExperimentalForeignApi::class)

package app.indelible.reader.pdf

import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.graphics.asComposeImageBitmap
import kotlinx.cinterop.CPointer
import kotlinx.cinterop.ExperimentalForeignApi
import kotlinx.cinterop.UByteVar
import kotlinx.cinterop.autoreleasepool
import kotlinx.cinterop.get
import kotlinx.cinterop.interpretCPointer
import kotlinx.cinterop.set
import org.jetbrains.skia.Bitmap
import org.jetbrains.skia.ColorAlphaType
import org.jetbrains.skia.ColorType
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

/** Draws [page] straight into a Skia bitmap's BGRA pixels at the bounded size for [widthPx], with [tone] applied. */
internal fun renderPage(
    page: PDFPage,
    size: PdfPageSize,
    widthPx: Int,
    tone: PdfTone,
): ImageBitmap {
    val (w, h) = renderSizeFor(size, widthPx)
    val bytes = bufferBytes(w, h)
    val bitmap = Bitmap()
    check(bitmap.allocPixels(ImageInfo(w, h, ColorType.BGRA_8888, ColorAlphaType.PREMUL))) { "no room for $w x $h" }
    val pixels = checkNotNull(interpretCPointer<UByteVar>(checkNotNull(bitmap.peekPixels()).addr))
    val space = CGColorSpaceCreateDeviceRGB()
    val info = CGImageAlphaInfo.kCGImageAlphaPremultipliedFirst.value or kCGBitmapByteOrder32Little
    val rowBytes = (w * BYTES_PER_PIXEL).toULong()
    val context = CGBitmapContextCreate(pixels, w.toULong(), h.toULong(), BITS_PER_COMPONENT, rowBytes, space, info)
    CGContextSetRGBFillColor(context, 1.0, 1.0, 1.0, 1.0)
    CGContextFillRect(context, CGRectMake(0.0, 0.0, w.toDouble(), h.toDouble()))
    // drawWithBox applies the crop box and /Rotate itself; only the scale is set here.
    CGContextScaleCTM(context, w / size.width, h / size.height)
    autoreleasepool { page.drawWithBox(kPDFDisplayBoxCropBox, toContext = context) }
    CGContextRelease(context)
    CGColorSpaceRelease(space)
    toneTable(tone)?.let { applyToneBgra(pixels, bytes, it) }
    bitmap.setImmutable()
    return bitmap.asComposeImageBitmap()
}

private fun applyToneBgra(
    pixels: CPointer<UByteVar>,
    bytes: Int,
    table: ByteArray,
) {
    var i = 0
    while (i < bytes) {
        pixels[i] = table[2 * LEVELS + pixels[i].toInt()].toUByte()
        pixels[i + 1] = table[LEVELS + pixels[i + 1].toInt()].toUByte()
        pixels[i + 2] = table[pixels[i + 2].toInt()].toUByte()
        i += BYTES_PER_PIXEL
    }
}
