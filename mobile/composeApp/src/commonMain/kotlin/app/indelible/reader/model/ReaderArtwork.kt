package app.indelible.reader.model

private const val HASH_PRIME = 31

// How far the chrome scrim takes to fade, in pixels scrolled. The wide, shallow
// ridges clear the chrome quickly; the two taller scenes need longer.
private const val SHALLOW_FIELD_VEIL_RANGE = 60
private const val TALL_FIELD_VEIL_RANGE = 130

/**
 * The three authored drawings that stand behind the masthead. One is chosen per
 * document and travels out of frame as the article scrolls.
 *
 * [veilRange] is the unitless scroll distance over which the chrome scrim fades
 * out, authored per drawing. It is not derived from the drawing's height: the
 * travel distance is measured from the rendered element at runtime, because the
 * frame width — and so the drawn height — depends on the device.
 */
enum class ReaderArtwork(
    val resourcePath: String,
    val veilRange: Int,
) {
    MISTY_RIDGES("files/reader_art_misty_ridges.svg", SHALLOW_FIELD_VEIL_RANGE),
    PINE_RIDGES("files/reader_art_pine_ridges.svg", TALL_FIELD_VEIL_RANGE),
    LAKESIDE_DUSK("files/reader_art_lakeside_dusk.svg", TALL_FIELD_VEIL_RANGE),
    ;

    companion object {
        /**
         * Deliberately hand-rolled rather than using [String.hashCode], which is not
         * guaranteed identical across Kotlin/JVM and Kotlin/Native — the same article
         * would otherwise show a different drawing on Android than on iOS. Mirrors
         * ThumbnailColor.forId in core/model/LibraryItem.kt.
         */
        fun forDocumentId(id: String): ReaderArtwork {
            var hash = 0
            for (ch in id) {
                hash = hash * HASH_PRIME + ch.code
            }
            val index = ((hash % entries.size) + entries.size) % entries.size
            return entries[index]
        }
    }
}
