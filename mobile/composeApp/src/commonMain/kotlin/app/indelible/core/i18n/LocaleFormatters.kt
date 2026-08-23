package app.indelible.core.i18n

import kotlinx.datetime.Instant

enum class LocalizedDateStyle {
    SHORT,
    MEDIUM,
    MONTH_DAY,
    WEEKDAY_MONTH_DAY,
}

expect object LocaleFormatters {
    fun date(
        instant: Instant,
        style: LocalizedDateStyle,
    ): String

    fun number(value: Long): String
}
