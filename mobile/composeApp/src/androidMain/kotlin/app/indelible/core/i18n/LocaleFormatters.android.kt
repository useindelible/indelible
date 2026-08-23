package app.indelible.core.i18n

import android.text.format.DateFormat
import kotlinx.datetime.Instant
import java.text.NumberFormat
import java.text.SimpleDateFormat
import java.util.Date
import java.util.Locale
import java.text.DateFormat as JavaDateFormat

actual object LocaleFormatters {
    actual fun date(
        instant: Instant,
        style: LocalizedDateStyle,
    ): String {
        val date = Date(instant.toEpochMilliseconds())
        return when (style) {
            LocalizedDateStyle.SHORT -> JavaDateFormat.getDateInstance(JavaDateFormat.SHORT).format(date)
            LocalizedDateStyle.MEDIUM -> JavaDateFormat.getDateInstance(JavaDateFormat.MEDIUM).format(date)
            LocalizedDateStyle.MONTH_DAY -> formatSkeleton(date, "MMMMd")
            LocalizedDateStyle.WEEKDAY_MONTH_DAY -> formatSkeleton(date, "EEEEMMMMd")
        }
    }

    actual fun number(value: Long): String = NumberFormat.getIntegerInstance().format(value)

    private fun formatSkeleton(
        date: Date,
        skeleton: String,
    ): String {
        val locale = Locale.getDefault()
        return SimpleDateFormat(DateFormat.getBestDateTimePattern(locale, skeleton), locale).format(date)
    }
}
