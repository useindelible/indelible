package app.indelible.reader.model

sealed interface HighlightTarget

data class HighlightRange(
    val startOffset: Long,
    val endOffset: Long,
    val chapter: String? = null,
) : HighlightTarget

/** Fractions of the rendered page box; [page] is 1-based and the box is the union of [rects]. */
data class PdfHighlightTarget(
    val page: Int,
    val x: Double,
    val y: Double,
    val width: Double,
    val height: Double,
    val textSnapshot: String,
    val rects: List<PdfRect>,
) : HighlightTarget
