package app.indelible.core.i18n

import android.content.res.Resources
import android.text.format.DateFormat
import kotlinx.datetime.Instant
import java.text.NumberFormat
import java.text.SimpleDateFormat
import java.util.Date
import java.util.Locale
import java.text.DateFormat as JavaDateFormat

actual object LocaleFormatters {
    private lateinit var resources: Resources

    internal fun initialize(resources: Resources) {
        this.resources = resources
    }

    actual fun date(
        instant: Instant,
        style: LocalizedDateStyle,
    ): String {
        val date = Date(instant.toEpochMilliseconds())
        val locale = currentLocale()
        return when (style) {
            LocalizedDateStyle.SHORT -> JavaDateFormat.getDateInstance(JavaDateFormat.SHORT, locale).format(date)
            LocalizedDateStyle.MEDIUM -> JavaDateFormat.getDateInstance(JavaDateFormat.MEDIUM, locale).format(date)
            LocalizedDateStyle.MONTH_DAY -> formatSkeleton(date, "MMMMd", locale)
            LocalizedDateStyle.WEEKDAY_MONTH_DAY -> formatSkeleton(date, "EEEEMMMMd", locale)
        }
    }

    actual fun number(value: Long): String = NumberFormat.getIntegerInstance(currentLocale()).format(value)

    private fun formatSkeleton(
        date: Date,
        skeleton: String,
        locale: Locale,
    ): String = SimpleDateFormat(DateFormat.getBestDateTimePattern(locale, skeleton), locale).format(date)

    private fun currentLocale(): Locale =
        checkNotNull(resources.configuration.locales[0]) {
            "Application resources must provide a locale"
        }
}
