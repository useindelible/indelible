package app.indelible.core.i18n

import java.text.DateFormat
import java.text.NumberFormat
import java.util.Date
import kotlinx.datetime.Instant

actual object LocaleFormatters {
    actual fun shortDate(instant: Instant): String =
        DateFormat.getDateInstance(DateFormat.SHORT).format(Date(instant.toEpochMilliseconds()))

    actual fun number(value: Long): String = NumberFormat.getIntegerInstance().format(value)
}
