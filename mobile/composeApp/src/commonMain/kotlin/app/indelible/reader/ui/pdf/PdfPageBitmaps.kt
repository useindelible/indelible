package app.indelible.reader.ui.pdf

import androidx.compose.ui.graphics.ImageBitmap
import app.indelible.reader.pdf.PdfDocument
import app.indelible.reader.pdf.PdfTone

internal const val PAGE_BITMAP_BUDGET_BYTES = 96L * 1024L * 1024L
private const val BYTES_PER_PIXEL = 4L

/** Rendered pages by page, width and tone, least recently used first out once [budgetBytes] is reached. */
internal class PdfPageBitmaps(
    private val document: PdfDocument,
    private val budgetBytes: Long = PAGE_BITMAP_BUDGET_BYTES,
) {
    private data class Key(
        val page: Int,
        val widthPx: Int,
        val tone: PdfTone,
    )

    private val cache = LinkedHashMap<Key, ImageBitmap>()
    private var usedBytes = 0L

    suspend fun bitmap(
        pageIndex: Int,
        widthPx: Int,
        tone: PdfTone,
    ): ImageBitmap {
        val key = Key(pageIndex, widthPx, tone)
        cache.remove(key)?.let { hit ->
            cache[key] = hit
            return hit
        }
        val bitmap = document.render(pageIndex, widthPx, tone)
        insert(key, bitmap)
        return bitmap
    }

    private fun insert(
        key: Key,
        bitmap: ImageBitmap,
    ) {
        cache.remove(key)?.let { usedBytes -= it.bytes() }
        val bytes = bitmap.bytes()
        while (cache.isNotEmpty() && usedBytes + bytes > budgetBytes) {
            val oldest = cache.keys.first()
            usedBytes -= cache.remove(oldest)?.bytes() ?: 0L
        }
        cache[key] = bitmap
        usedBytes += bytes
    }

    private fun ImageBitmap.bytes(): Long = width.toLong() * height.toLong() * BYTES_PER_PIXEL
}
