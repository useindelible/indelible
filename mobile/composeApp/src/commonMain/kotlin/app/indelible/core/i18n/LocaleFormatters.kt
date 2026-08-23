package app.indelible.core.i18n

import kotlinx.datetime.Instant

expect object LocaleFormatters {
    fun shortDate(instant: Instant): String

    fun number(value: Long): String
}
