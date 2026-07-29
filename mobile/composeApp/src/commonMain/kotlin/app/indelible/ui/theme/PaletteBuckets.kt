package app.indelible.ui.theme

import kotlin.math.abs

const val PALETTE_BUCKET_SLOTS = 5

// Colour-name prefixes per palette slot, in slot order: each entry's list index is
// its slot in the 5-colour banner/tag palette (0=blue, 1=green, 2=purple, 3=pink,
// 4=yellow). Shared by tag dots and collection/smart-list banners so both derive the
// same colour for a given name.
private val PALETTE_BUCKET_PREFIXES: List<List<String>> =
    listOf(
        listOf("blue", "cyan", "indigo"),
        listOf("green", "teal", "lime"),
        listOf("purple", "violet"),
        listOf("pink", "red", "rose", "magenta"),
        listOf("yellow", "orange", "amber"),
    )

/**
 * Resolves a stable palette-slot index for a coloured entity. Recognised colour
 * names map to their documented slot; anything else (or a null colour) falls back
 * to a hash of the colour string, then the entity id, so the choice stays
 * deterministic without clustering unknown colours on one slot.
 */
internal fun paletteBucketIndex(
    color: String?,
    id: String,
    slots: Int = PALETTE_BUCKET_SLOTS,
): Int {
    val normalized = color?.lowercase()?.trim() ?: return abs(id.hashCode()) % slots
    val slot = PALETTE_BUCKET_PREFIXES.indexOfFirst { prefixes -> prefixes.any { normalized.startsWith(it) } }
    return if (slot >= 0) slot else abs(normalized.hashCode()) % slots
}
