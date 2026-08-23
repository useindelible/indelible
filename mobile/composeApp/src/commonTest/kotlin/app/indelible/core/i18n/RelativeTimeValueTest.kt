package app.indelible.core.i18n

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlinx.datetime.Instant
import kotlin.time.Duration.Companion.days
import kotlin.time.Duration.Companion.minutes

class RelativeTimeValueTest {
    private val now = Instant.parse("2026-08-23T12:00:00Z")

    @Test
    fun selectsSemanticUnitsWithoutEmbeddingLanguage() {
        assertEquals(RelativeTimeValue(5, RelativeTimeUnit.MINUTE, false), relativeTimeValue(now - 5.minutes, now))
        assertEquals(RelativeTimeValue(2, RelativeTimeUnit.MONTH, false), relativeTimeValue(now - 61.days, now))
    }
}
