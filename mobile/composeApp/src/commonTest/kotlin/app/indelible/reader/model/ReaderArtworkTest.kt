package app.indelible.reader.model

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class ReaderArtworkTest {
    @Test
    fun sameDocumentIdAlwaysPicksTheSameDrawing() {
        val id = "doc_01JQ8Z3K7M4N5P6Q7R8S9T0U1V"
        val first = ReaderArtwork.forDocumentId(id)
        repeat(50) {
            assertEquals(first, ReaderArtwork.forDocumentId(id))
        }
    }

    @Test
    fun differentDocumentIdsSpreadAcrossAllThreeDrawings() {
        val seen =
            (1..300)
                .map { ReaderArtwork.forDocumentId("doc_$it") }
                .toSet()
        assertEquals(ReaderArtwork.entries.toSet(), seen)
    }

    @Test
    fun emptyAndShortIdsStillResolve() {
        // The hash starts at 0, so an empty id must not divide by zero or index out of range.
        assertTrue(ReaderArtwork.forDocumentId("") in ReaderArtwork.entries)
        assertTrue(ReaderArtwork.forDocumentId("a") in ReaderArtwork.entries)
    }

    @Test
    fun idsThatOverflowTheHashStillResolve() {
        // hash * 31 overflows Int well before a long id is consumed; the guarded modulo
        // has to survive a negative accumulator rather than throwing on a negative index.
        val long = "d".repeat(500)
        assertTrue(ReaderArtwork.forDocumentId(long) in ReaderArtwork.entries)
        val overflowing = "￿".repeat(64)
        assertTrue(ReaderArtwork.forDocumentId(overflowing) in ReaderArtwork.entries)
    }

    @Test
    fun everyDrawingDeclaresAResourceAndVeilRange() {
        ReaderArtwork.entries.forEach { art ->
            assertTrue(art.resourcePath.startsWith("files/"), "${art.name} resource path")
            assertTrue(art.resourcePath.endsWith(".svg"), "${art.name} resource extension")
            assertTrue(art.veilRange > 0, "${art.name} veil range")
        }
    }
}
