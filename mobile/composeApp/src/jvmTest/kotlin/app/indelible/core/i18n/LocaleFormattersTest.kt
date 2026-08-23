package app.indelible.core.i18n

import kotlinx.datetime.Instant
import java.util.Locale
import java.util.TimeZone
import kotlin.test.Test
import kotlin.test.assertEquals

class LocaleFormattersTest {
    @Test
    fun numberUsesTheDefaultLocale() {
        withDefaultLocale(Locale.US) {
            assertEquals("1,234", LocaleFormatters.number(1234))
        }
    }

    @Test
    fun datesUseTheDefaultLocaleAndRequestedStyle() {
        val instant = Instant.parse("2025-01-02T12:00:00Z")

        withDefaultLocale(Locale.US) {
            assertEquals("1/2/25", LocaleFormatters.date(instant, LocalizedDateStyle.SHORT))
            assertEquals("Jan 2, 2025", LocaleFormatters.date(instant, LocalizedDateStyle.MEDIUM))
            assertEquals("January 2", LocaleFormatters.date(instant, LocalizedDateStyle.MONTH_DAY))
            assertEquals("Thursday, January 2", LocaleFormatters.date(instant, LocalizedDateStyle.WEEKDAY_MONTH_DAY))
        }

        withDefaultLocale(Locale.FRANCE) {
            assertEquals("02/01/2025", LocaleFormatters.date(instant, LocalizedDateStyle.SHORT))
            assertEquals("2 janv. 2025", LocaleFormatters.date(instant, LocalizedDateStyle.MEDIUM))
            assertEquals("2 janvier", LocaleFormatters.date(instant, LocalizedDateStyle.MONTH_DAY))
            assertEquals("jeudi 2 janvier", LocaleFormatters.date(instant, LocalizedDateStyle.WEEKDAY_MONTH_DAY))
        }
    }

    private inline fun withDefaultLocale(
        locale: Locale,
        block: () -> Unit,
    ) {
        val previous = Locale.getDefault()
        val previousTimeZone = TimeZone.getDefault()
        try {
            Locale.setDefault(locale)
            TimeZone.setDefault(TimeZone.getTimeZone("UTC"))
            block()
        } finally {
            Locale.setDefault(previous)
            TimeZone.setDefault(previousTimeZone)
        }
    }
}
