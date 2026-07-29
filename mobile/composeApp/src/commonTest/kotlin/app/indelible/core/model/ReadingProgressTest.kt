package app.indelible.core.model

import kotlin.test.Test
import kotlin.test.assertEquals

class ReadingProgressTest {
    @Test
    fun formatReadingTimeUsesMinutesUnderAnHour() {
        assertEquals("8 MIN", formatReadingTime(8))
        assertEquals("0 MIN", formatReadingTime(0))
        assertEquals("59 MIN", formatReadingTime(59))
    }

    @Test
    fun formatReadingTimeRollsUpIntoHours() {
        assertEquals("1 HR", formatReadingTime(60))
        assertEquals("1 HR 30 MIN", formatReadingTime(90))
        assertEquals("2 HR", formatReadingTime(120))
        assertEquals("2 HR 5 MIN", formatReadingTime(125))
    }
}
