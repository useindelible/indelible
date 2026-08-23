package app.indelible.core.i18n

import java.util.Locale
import kotlin.test.Test
import kotlin.test.assertEquals

class LocaleFormattersTest {
    @Test
    fun numberUsesTheDefaultLocale() {
        withDefaultLocale(Locale.US) {
            assertEquals("1,234", LocaleFormatters.number(1234))
        }
    }

    private inline fun withDefaultLocale(
        locale: Locale,
        block: () -> Unit,
    ) {
        val previous = Locale.getDefault()
        try {
            Locale.setDefault(locale)
            block()
        } finally {
            Locale.setDefault(previous)
        }
    }
}
