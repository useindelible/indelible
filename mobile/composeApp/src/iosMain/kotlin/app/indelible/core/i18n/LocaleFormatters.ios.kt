package app.indelible.core.i18n

import kotlinx.datetime.Instant
import platform.Foundation.NSDate
import platform.Foundation.NSDateFormatter
import platform.Foundation.NSDateFormatterMediumStyle
import platform.Foundation.NSDateFormatterShortStyle
import platform.Foundation.NSNumber
import platform.Foundation.NSNumberFormatter

actual object LocaleFormatters {
    actual fun date(
        instant: Instant,
        style: LocalizedDateStyle,
    ): String {
        val formatter = NSDateFormatter()
        when (style) {
            LocalizedDateStyle.SHORT -> formatter.dateStyle = NSDateFormatterShortStyle
            LocalizedDateStyle.MEDIUM -> formatter.dateStyle = NSDateFormatterMediumStyle
            LocalizedDateStyle.MONTH_DAY -> formatter.setLocalizedDateFormatFromTemplate("MMMMd")
            LocalizedDateStyle.WEEKDAY_MONTH_DAY -> formatter.setLocalizedDateFormatFromTemplate("EEEEMMMMd")
        }
        return formatter.stringFromDate(instant.toNSDate())
    }

    actual fun number(value: Long): String =
        NSNumberFormatter().stringFromNumber(NSNumber(longLong = value)) ?: value.toString()
}

private fun Instant.toNSDate(): NSDate = NSDate(
    timeIntervalSinceReferenceDate =
        toEpochMilliseconds().toDouble() / 1_000 - APPLE_REFERENCE_DATE_UNIX_SECONDS,
)

private const val APPLE_REFERENCE_DATE_UNIX_SECONDS = 978_307_200.0
