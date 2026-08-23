package app.indelible.core.i18n

import kotlinx.datetime.Instant
import java.text.DateFormat
import java.text.NumberFormat
import java.text.SimpleDateFormat
import java.util.Date
import java.util.Locale

actual object LocaleFormatters {
    actual fun date(
        instant: Instant,
        style: LocalizedDateStyle,
    ): String {
        val date = Date(instant.toEpochMilliseconds())
        return when (style) {
            LocalizedDateStyle.SHORT -> DateFormat.getDateInstance(DateFormat.SHORT).format(date)
            LocalizedDateStyle.MEDIUM -> DateFormat.getDateInstance(DateFormat.MEDIUM).format(date)
            LocalizedDateStyle.MONTH_DAY -> SimpleDateFormat(monthDayPattern(), Locale.getDefault()).format(date)
            LocalizedDateStyle.WEEKDAY_MONTH_DAY ->
                SimpleDateFormat(weekdayMonthDayPattern(), Locale.getDefault()).format(date)
        }
    }

    actual fun number(value: Long): String = NumberFormat.getIntegerInstance().format(value)

    private fun monthDayPattern(): String =
        when (Locale.getDefault().language) {
            Locale.FRENCH.language -> "d MMMM"
            else -> "MMMM d"
        }

    private fun weekdayMonthDayPattern(): String =
        when (Locale.getDefault().language) {
            Locale.FRENCH.language -> "EEEE d MMMM"
            else -> "EEEE, MMMM d"
        }
}
