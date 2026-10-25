package app.indelible.reader.pdf

import android.os.ParcelFileDescriptor
import io.legere.pdfiumandroid.suspend.PdfDocumentKt
import io.legere.pdfiumandroid.suspend.PdfiumCoreKt
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import java.io.File
import java.io.IOException

actual object PdfEngine {
    // pdfium is not thread-safe, so every document shares this serial dispatcher.
    private val dispatcher = Dispatchers.IO.limitedParallelism(1)
    private val core by lazy { PdfiumCoreKt(dispatcher) }

    actual suspend fun open(path: String): PdfDocument =
        withContext(dispatcher) {
            val descriptor =
                try {
                    ParcelFileDescriptor.open(File(path), ParcelFileDescriptor.MODE_READ_ONLY)
                } catch (error: IOException) {
                    throw PdfOpenException("pdfium cannot read $path", error)
                }
            val document =
                runCatching { core.newDocument(descriptor) }.getOrElse { error ->
                    descriptor.close()
                    throw error.asOpenFailure(path)
                }
            // The document owns the descriptor from here on; closing it closes both.
            runCatching { PdfiumDocument.open(document, dispatcher) }.getOrElse { error ->
                document.safeClose()
                throw error.asOpenFailure(path)
            }
        }

    private fun Throwable.asOpenFailure(path: String): Throwable =
        if (this is CancellationException) this else PdfOpenException("pdfium cannot open $path", this)
}

internal fun PdfDocumentKt.safeClose() {
    runCatching { close() }
}
