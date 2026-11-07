@file:OptIn(ExperimentalForeignApi::class)

package app.indelible.reader.pdf

import kotlinx.cinterop.ExperimentalForeignApi
import kotlinx.cinterop.autoreleasepool
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import platform.Foundation.NSURL
import platform.PDFKit.PDFDocument

actual object PdfEngine {
    // PDFKit is not thread-safe, so every document shares this serial dispatcher: one call runs at a time.
    private val dispatcher = Dispatchers.Default.limitedParallelism(1)

    actual suspend fun open(path: String): PdfDocument =
        withContext(dispatcher) {
            autoreleasepool {
                val document =
                    runCatching { PDFDocument(uRL = NSURL.fileURLWithPath(path)) }.getOrElse { error ->
                        throw PdfOpenException("PDFKit cannot open $path", error)
                    }
                if (document == null || document.isLocked() || document.pageCount == 0uL) {
                    throw PdfOpenException("PDFKit cannot open $path")
                }
                PdfKitDocument(document, dispatcher)
            }
        }
}
