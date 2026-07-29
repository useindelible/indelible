package app.indelible.reader.ui

import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.Stable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import app.indelible.reader.model.DataPanel

/**
 * How far the page must move before it counts as a deliberate scroll rather than
 * jitter — a thumb settling at the end of a flick, or a layout nudge. The same
 * threshold serves both directions: down retires the chrome, up recalls it.
 */
internal const val READER_SCROLL_DELTA_PX = 12f

/**
 * Chrome visibility for the reader.
 *
 * The furniture follows the scroll direction and nothing else: reading forward
 * hides it, turning back brings it back, and standing still leaves it wherever
 * it is. There is no idle timer — chrome never disappears while you linger.
 */
@Stable
internal class ReaderChromeState {
    var immersive by mutableStateOf(false)
        private set

    private var lastOffset: Float? = null

    /**
     * Reading forward hides the furniture; turning back calls it. The first event
     * only primes the baseline: a restored reading position arrives as one large
     * offset jump and must not read as a downward flick. While [suppressed] — an
     * open sheet, a live selection — scrolling never retires the chrome, since the
     * open surface's dismiss target would be stranded.
     */
    fun onScroll(
        offset: Float,
        suppressed: Boolean,
    ) {
        val last = lastOffset
        lastOffset = offset
        if (last == null) return
        val delta = offset - last
        when {
            delta <= -READER_SCROLL_DELTA_PX -> reveal()
            delta >= READER_SCROLL_DELTA_PX && !suppressed -> retire()
        }
    }

    /** A tap, a selection, a highlight — anything that reaches for the chrome. */
    fun reveal() {
        immersive = false
    }

    fun retire() {
        immersive = true
    }
}

/**
 * Whether something on screen is a deliberate stop rather than a rest. Any of these
 * would have its dismiss target stranded if the furniture left underneath it.
 */
internal fun isChromeSuppressed(
    activePanel: DataPanel,
    hasSelection: Boolean,
    hasTappedHighlight: Boolean,
    hasOpenTagSheet: Boolean,
): Boolean = activePanel != DataPanel.NONE || hasSelection || hasTappedHighlight || hasOpenTagSheet

/**
 * Reveals and holds the chrome while a sheet, selection, or highlight is open,
 * so the open surface's dismiss target is never stranded.
 */
@Composable
internal fun ReaderChromeSuppressionEffect(
    state: ReaderChromeState,
    suppressed: Boolean,
) {
    LaunchedEffect(suppressed) {
        if (suppressed) state.reveal()
    }
}
