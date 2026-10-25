package app.indelible.reader.pdf

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class PdfSelectionTextTest {
    private val text = "The quick brown fox\njumps over"

    @Test
    fun partialWordsGrowToWholeWords() {
        assertEquals(4..18, snapToWords(text, 6, 18))
    }

    @Test
    fun wholeWordsStay() {
        assertEquals(4..18, snapToWords(text, 4, 19))
    }

    @Test
    fun selectionAcrossLinesSnapsBothEnds() {
        assertEquals(10..24, snapToWords(text, 12, 22))
    }

    @Test
    fun rangeOutsideTheTextIsClamped() {
        assertEquals(26..29, snapToWords(text, 28, 99))
        assertEquals(0..2, snapToWords(text, -5, 2))
    }

    @Test
    fun emptyTextGivesAnEmptyRange() {
        assertEquals(IntRange.EMPTY, snapToWords("", 0, 0))
    }

    @Test
    fun dragStartingOnASpaceBeginsAtTheNextWord() {
        assertEquals(4..8, snapToWords(text, 3, 6))
    }

    @Test
    fun trailingWhitespaceIsLeftOut() {
        assertEquals(16..18, snapToWords(text, 16, 20))
    }

    @Test
    fun whitespaceOnlySelectionIsEmpty() {
        assertTrue(snapToWords(text, 19, 20).isEmpty())
    }

    @Test
    fun ideographsAreWordsOfTheirOwn() {
        assertEquals(1..1, snapToWords("日本語の文章", 1, 2))
        assertEquals(1..3, snapToWords("日本語の文章", 1, 4))
    }

    @Test
    fun latinRunStopsAtAnIdeograph() {
        assertEquals(0..2, snapToWords("abc日本", 1, 2))
        assertEquals(3..3, snapToWords("abc日本", 3, 4))
    }
}
