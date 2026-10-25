package app.indelible.reader.pdf

import kotlin.test.Test
import kotlin.test.assertEquals

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
}
