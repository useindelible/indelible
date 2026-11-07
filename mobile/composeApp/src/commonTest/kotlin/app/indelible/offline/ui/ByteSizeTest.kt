package app.indelible.offline.ui

import kotlin.test.Test
import kotlin.test.assertEquals

class ByteSizeTest {
    @Test
    fun smallSizesRoundUpToWholeKilobytes() {
        assertEquals(DisplaySize(0.0, SizeUnit.KB), displaySize(0))
        assertEquals(DisplaySize(1.0, SizeUnit.KB), displaySize(1))
        assertEquals(DisplaySize(2.0, SizeUnit.KB), displaySize(1_025))
    }

    @Test
    fun largerSizesUseTheLargestWholeUnit() {
        assertEquals(DisplaySize(1.5, SizeUnit.MB), displaySize(3L * 512 * 1024))
        assertEquals(DisplaySize(1.0, SizeUnit.GB), displaySize(1024L * 1024 * 1024))
        assertEquals(DisplaySize(0.25 * 1024, SizeUnit.MB), displaySize(256L * 1024 * 1024))
    }
}
