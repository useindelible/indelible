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

// Read once per drawing per process. There are only three, ~135KB total, and the
// alternative is re-reading on every paper switch and typography change, each of
// which rebuilds the document.
private val artworkCache = mutableMapOf<ReaderArtwork, String>()

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
