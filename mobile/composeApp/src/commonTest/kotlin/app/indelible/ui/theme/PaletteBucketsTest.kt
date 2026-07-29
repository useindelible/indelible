package app.indelible.ui.theme

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class PaletteBucketsTest {
    @Test
    fun named_colors_map_to_their_documented_slot() {
        assertEquals(0, paletteBucketIndex("blue", "id"))
        assertEquals(0, paletteBucketIndex("cyan", "id"))
        assertEquals(0, paletteBucketIndex("indigo", "id"))
        assertEquals(1, paletteBucketIndex("green", "id"))
        assertEquals(1, paletteBucketIndex("teal", "id"))
        assertEquals(2, paletteBucketIndex("purple", "id"))
        assertEquals(2, paletteBucketIndex("violet", "id"))
        assertEquals(3, paletteBucketIndex("pink", "id"))
        assertEquals(3, paletteBucketIndex("red", "id"))
        assertEquals(3, paletteBucketIndex("magenta", "id"))
        assertEquals(4, paletteBucketIndex("yellow", "id"))
        assertEquals(4, paletteBucketIndex("orange", "id"))
        assertEquals(4, paletteBucketIndex("amber", "id"))
    }

    @Test
    fun matching_is_case_insensitive_trimmed_and_prefix_based() {
        assertEquals(0, paletteBucketIndex("  BLUE  ", "id"))
        assertEquals(0, paletteBucketIndex("Blue-500", "id"))
        assertEquals(1, paletteBucketIndex("GreenForest", "id"))
    }

    @Test
    fun unknown_color_falls_back_to_a_deterministic_in_range_hash() {
        val first = paletteBucketIndex("chartreuse", "id")
        val second = paletteBucketIndex("chartreuse", "other-id")
        assertEquals(first, second, "unknown colour should hash on the colour, not the id")
        assertTrue(first in 0 until PALETTE_BUCKET_SLOTS)
    }

    @Test
    fun null_color_falls_back_to_a_deterministic_id_hash() {
        val a = paletteBucketIndex(null, "collection-123")
        val b = paletteBucketIndex(null, "collection-123")
        assertEquals(a, b, "same id must always resolve to the same slot")
        assertTrue(a in 0 until PALETTE_BUCKET_SLOTS)
    }

    @Test
    fun blank_color_is_treated_as_unknown_and_uses_the_color_hash() {
        // Blank trims to empty, which matches no prefix, so it hashes the (empty) colour
        // string rather than the id.
        assertEquals(paletteBucketIndex("   ", "id-a"), paletteBucketIndex("   ", "id-b"))
    }

    @Test
    fun slots_argument_bounds_the_result() {
        repeat(20) { i ->
            assertTrue(paletteBucketIndex(null, "id-$i", slots = 3) in 0 until 3)
        }
    }
}
