package app.indelible.library.ui

import app.indelible.core.model.LibraryCounts
import app.indelible.library.viewmodel.LibraryScope
import kotlin.test.Test
import kotlin.test.assertEquals

class LibraryScreenTest {
    @Test
    fun triage_scope_keeps_its_zero_count_visible() {
        val emptyCounts =
            LibraryCounts(
                total = 0,
                unread = 0,
                reading = 0,
                done = 0,
                byItemType = emptyMap(),
            )

        assertEquals(
            0,
            scopeCount(
                scope = LibraryScope.Triage,
                collections = emptyList(),
                counts = emptyCounts,
            ),
        )
    }
}
