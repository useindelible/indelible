package app.indelible.reader.pdf

import android.os.ParcelFileDescriptor
import io.legere.pdfiumandroid.suspend.PdfiumCoreKt
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
            try {
                PdfiumDocument.open(core.newDocument(descriptor), descriptor, dispatcher)
            } catch (error: IOException) {
                descriptor.close()
                throw PdfOpenException("pdfium cannot open $path", error)
            } catch (error: IllegalStateException) {
                descriptor.close()
                throw PdfOpenException("pdfium cannot open $path", error)
            }
        }
}
