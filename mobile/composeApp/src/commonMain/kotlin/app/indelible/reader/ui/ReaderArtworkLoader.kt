package app.indelible.reader.ui

import androidx.compose.runtime.Composable
import androidx.compose.runtime.produceState
import androidx.compose.runtime.remember
import app.indelible.reader.model.ReaderArtwork
import indelible.composeapp.generated.resources.Res

data class LoadedReaderArtwork(
    val artwork: ReaderArtwork,
    val svg: String,
)

/** What a reader page embeds besides its text: the drawing behind the masthead and the font faces. */
data class ReaderAssets(
    val artwork: LoadedReaderArtwork?,
    val fontFaceCss: String,
) {
    companion object {
        val NONE = ReaderAssets(artwork = null, fontFaceCss = "")
    }
}

// Read once per drawing per process. There are only three, ~135KB total, and the
// alternative is re-reading on every paper switch and typography change, each of
// which rebuilds the document.
private val artworkCache = mutableMapOf<ReaderArtwork, String>()
private var fontCssCache: String? = null

/**
 * Loads the drawing for [documentId]. Returns null until the read completes; the
 * caller must hold back the web view rather than rendering with an empty aura and
 * swapping later — the swap would change the HTML that keys the view's `remember`,
 * reloading the document and losing scroll position.
 */
@Composable
internal fun rememberReaderArtwork(documentId: String): LoadedReaderArtwork? {
    val artwork = remember(documentId) { ReaderArtwork.forDocumentId(documentId) }
    val cached = artworkCache[artwork]
    return produceState<LoadedReaderArtwork?>(
        initialValue = cached?.let { LoadedReaderArtwork(artwork, it) },
        key1 = artwork,
    ) {
        val svg = artworkCache.getOrPut(artwork) { Res.readBytes(artwork.resourcePath).decodeToString() }
        value = LoadedReaderArtwork(artwork, svg)
    }.value
}

/** The bundled font faces, read once per process; null until read, under the same hold-back rule. */
@Composable
internal fun rememberReaderFontCss(): String? =
    produceState(initialValue = fontCssCache) {
        value = fontCssCache ?: loadReaderFontCss().also { fontCssCache = it }
    }.value
