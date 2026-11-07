package app.indelible.reader.viewmodel

import kotlin.test.Test
import kotlin.test.assertEquals

class EpubChaptersTest {
    @Test
    fun sequenceFollowsTheServerChapterList() {
        assertEquals(listOf(0, 2, 5), chapterSequence(bookToc()).map { it.spine })
        assertEquals(listOf("c0.xhtml", "c2.xhtml", "c5.xhtml"), chapterSequence(bookToc()).map { it.href })
    }

    @Test
    fun keyIsTheFirstTocEntryForTheSpine() {
        assertEquals("e5", chapterSequence(bookToc())[2].key)
    }

    @Test
    fun unlistedSpineIsKeyedBySpineIndex() {
        assertEquals("spine:2", chapterSequence(bookToc())[1].key)
    }

    @Test
    fun titleFallsBackToServerThenOrdinal() {
        val toc = bookToc(chapters = listOf(storedChapter(0), storedChapter(2, "Server two"), storedChapter(3)))

        assertEquals(listOf("Te0", "Server two", null), chapterSequence(toc).map { it.title })
    }

    @Test
    fun bookPercentCountsWholeChaptersBeforeTheCurrentOne() {
        assertEquals(0f, bookPercent(ordinal = 0, chapterPercent = 0f, count = 4))
        assertEquals(37.5f, bookPercent(ordinal = 1, chapterPercent = 50f, count = 4))
        assertEquals(100f, bookPercent(ordinal = 3, chapterPercent = 100f, count = 4))
    }
}
