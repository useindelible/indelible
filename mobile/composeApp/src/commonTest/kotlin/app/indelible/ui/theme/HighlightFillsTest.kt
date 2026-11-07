package app.indelible.ui.theme

import androidx.compose.ui.graphics.Color
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull

class HighlightFillsTest {
    private val fills =
        HighlightFills(
            yellow = Color(0xFF010101),
            blue = Color(0xFF020202),
            green = Color(0xFF030303),
            pink = Color(0xFF040404),
            purple = Color(0xFF050505),
        )

    @Test
    fun forApiColorMapsEveryEnumValueAndRejectsBookmark() {
        assertEquals(fills.yellow, fills.forApiColor("yellow"))
        assertEquals(fills.blue, fills.forApiColor("blue"))
        assertEquals(fills.green, fills.forApiColor("green"))
        assertEquals(fills.pink, fills.forApiColor("pink"))
        assertEquals(fills.purple, fills.forApiColor("purple"))
        assertNull(fills.forApiColor("bookmark"))
    }
}
