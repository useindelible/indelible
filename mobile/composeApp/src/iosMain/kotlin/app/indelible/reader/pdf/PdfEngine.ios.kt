@file:OptIn(ExperimentalForeignApi::class)

package app.indelible.reader.pdf

import kotlinx.cinterop.ExperimentalForeignApi
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import platform.Foundation.NSURL
import platform.PDFKit.PDFDocument

actual object PdfEngine {
    // PDFKit objects are touched from one thread only, so every document shares this serial dispatcher.
    private val dispatcher = Dispatchers.Default.limitedParallelism(1)

    actual suspend fun open(path: String): PdfDocument =
        withContext(dispatcher) {
            val document = runCatching { PDFDocument(uRL = NSURL.fileURLWithPath(path)) }.getOrNull()
            if (document == null || document.isLocked() || document.pageCount == 0uL) {
                throw PdfOpenException("PDFKit cannot open $path")
            }
            PdfKitDocument(document, dispatcher)
        }
}
