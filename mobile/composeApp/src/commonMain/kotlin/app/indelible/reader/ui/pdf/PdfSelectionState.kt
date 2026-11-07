package app.indelible.reader.ui.pdf

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.runtime.snapshotFlow
import app.indelible.reader.pdf.NormalizedPoint
import app.indelible.reader.pdf.NormalizedRect
import app.indelible.reader.pdf.PdfDocument
import app.indelible.reader.pdf.PdfSelection
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.launch

/**
 * One long-press drag at a time: the engine is asked for every new focus while the finger moves, and once more
 * for the final points on release; only that last answer is reported, and only if no newer gesture began.
 */
internal class PdfSelectionState(
    private val document: PdfDocument,
    private val scope: CoroutineScope,
) {
    private data class Drag(
        val page: Int,
        val anchor: NormalizedPoint,
        val focus: NormalizedPoint,
        val id: Int,
    )

    private var drag by mutableStateOf<Drag?>(null)
    private var gesture = 0
    private var shown by mutableStateOf<Pair<Int, List<NormalizedRect>>?>(null)

    fun rectsOn(page: Int): List<NormalizedRect> = shown?.takeIf { it.first == page }?.second.orEmpty()

    fun start(
        page: Int,
        point: NormalizedPoint,
    ) {
        gesture++
        shown = null
        drag = Drag(page, point, point, gesture)
    }

    fun move(point: NormalizedPoint) {
        drag = drag?.copy(focus = point)
    }

    fun cancel() {
        gesture++
        drag = null
        shown = null
    }

    /** Drops whatever is shown or pending; true when there was something to drop. */
    fun clear(): Boolean {
        val had = shown != null || drag != null
        cancel()
        return had
    }

    fun end(onResult: (PdfSelection?) -> Unit) {
        val released = drag ?: return
        drag = null
        scope.launch {
            val selection = document.trySelect(released)
            if (released.id != gesture) return@launch
            shown = selection?.let { released.page to it.rects }
            onResult(selection)
        }
    }

    suspend fun followDrags() {
        snapshotFlow { drag }.collectLatest { live ->
            if (live != null) shown = live.page to document.trySelect(live)?.rects.orEmpty()
        }
    }

    private suspend fun PdfDocument.trySelect(drag: Drag): PdfSelection? =
        runCatching { select(drag.page, drag.anchor, drag.focus) }.getOrElse { error ->
            if (error is CancellationException) throw error
            null
        }
}
