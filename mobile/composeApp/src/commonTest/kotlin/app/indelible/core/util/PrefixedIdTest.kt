package app.indelible.core.util

import kotlinx.datetime.Clock
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class PrefixedIdTest {
    private val uuidRegex =
        Regex("^[0-9a-f]{8}-[0-9a-f]{4}-7[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$")

    @Test
    fun uuidV7HasCanonicalLowercaseHyphenatedForm() {
        assertTrue(uuidRegex.matches(uuidV7()))
    }

    @Test
    fun uuidV7HasVersionNibbleSeven() {
        assertEquals('7', uuidV7()[14])
    }

    @Test
    fun uuidV7HasVariantBitsTen() {
        assertTrue(uuidV7()[19] in "89ab")
    }

    @Test
    fun uuidV7TimestampPrefixIsMonotonicAcrossCalls() {
        val first = uuidV7()
        val startMillis = Clock.System.now().toEpochMilliseconds()
        // Busy-wait on the real wall clock: kotlinx.datetime has no portable sleep, and
        // runTest's virtual clock would not advance Clock.System.
        while (Clock.System.now().toEpochMilliseconds() < startMillis + 2) {
            // intentionally empty
        }
        val second = uuidV7()

        assertTrue(timestampPrefix(first) <= timestampPrefix(second))
    }

    @Test
    fun readingEventIdHasPrefixAndValidUuid() {
        val id = readingEventId()
        assertTrue(id.startsWith("rev_"))
        assertTrue(uuidRegex.matches(id.removePrefix("rev_")))
    }

    @Test
    fun highlightClientIdHasPrefixAndValidUuid() {
        val id = highlightClientId()
        assertTrue(id.startsWith("hlt_"))
        assertTrue(uuidRegex.matches(id.removePrefix("hlt_")))
    }

    @Test
    fun clientIdHasPrefixAndValidUuid() {
        val id = clientId()
        assertTrue(id.startsWith("cli_"))
        assertTrue(uuidRegex.matches(id.removePrefix("cli_")))
    }

    private fun timestampPrefix(uuid: String): String = uuid.replace("-", "").substring(0, 12)
}
