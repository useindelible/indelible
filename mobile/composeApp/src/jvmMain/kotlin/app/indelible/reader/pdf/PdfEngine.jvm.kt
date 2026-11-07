package app.indelible.reader.pdf

actual object PdfEngine {
    actual suspend fun open(path: String): PdfDocument = throw PdfOpenException("No PDF engine on desktop")
}
