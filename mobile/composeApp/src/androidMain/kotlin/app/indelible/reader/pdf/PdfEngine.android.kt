package app.indelible.reader.pdf

actual object PdfEngine {
    actual suspend fun open(path: String): PdfDocument = throw PdfOpenException("PDF rendering is not available")
}
