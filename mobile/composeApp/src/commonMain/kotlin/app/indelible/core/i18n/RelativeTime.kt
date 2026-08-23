package app.indelible.core.i18n

import androidx.compose.runtime.Composable
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.relative_time_day
import indelible.composeapp.generated.resources.relative_time_future
import indelible.composeapp.generated.resources.relative_time_hour
import indelible.composeapp.generated.resources.relative_time_minute
import indelible.composeapp.generated.resources.relative_time_month
import indelible.composeapp.generated.resources.relative_time_now
import indelible.composeapp.generated.resources.relative_time_past
import indelible.composeapp.generated.resources.relative_time_week
import indelible.composeapp.generated.resources.relative_time_year
import kotlin.math.absoluteValue
import kotlinx.datetime.Instant
import org.jetbrains.compose.resources.pluralStringResource
import org.jetbrains.compose.resources.stringResource

enum class RelativeTimeUnit {
    NOW,
    MINUTE,
    HOUR,
    DAY,
    WEEK,
    MONTH,
    YEAR,
}

data class RelativeTimeValue(
    val amount: Int,
    val unit: RelativeTimeUnit,
    val future: Boolean,
)

fun relativeTimeValue(
    instant: Instant,
    now: Instant,
): RelativeTimeValue {
    val seconds = (instant - now).inWholeSeconds
    val absoluteSeconds = seconds.absoluteValue
    val (amount, unit) =
        when {
            absoluteSeconds < MINUTE_SECONDS -> 0L to RelativeTimeUnit.NOW
            absoluteSeconds < HOUR_SECONDS -> absoluteSeconds / MINUTE_SECONDS to RelativeTimeUnit.MINUTE
            absoluteSeconds < DAY_SECONDS -> absoluteSeconds / HOUR_SECONDS to RelativeTimeUnit.HOUR
            absoluteSeconds < WEEK_SECONDS -> absoluteSeconds / DAY_SECONDS to RelativeTimeUnit.DAY
            absoluteSeconds < MONTH_SECONDS -> absoluteSeconds / WEEK_SECONDS to RelativeTimeUnit.WEEK
            absoluteSeconds < YEAR_SECONDS -> absoluteSeconds / MONTH_SECONDS to RelativeTimeUnit.MONTH
            else -> absoluteSeconds / YEAR_SECONDS to RelativeTimeUnit.YEAR
        }

    return RelativeTimeValue(amount.coerceAtMost(Int.MAX_VALUE.toLong()).toInt(), unit, seconds > 0)
}

@Composable
fun relativeTimeText(
    instant: Instant,
    now: Instant,
): String {
    val value = relativeTimeValue(instant, now)
    if (value.unit == RelativeTimeUnit.NOW) return stringResource(Res.string.relative_time_now)

    val duration =
        when (value.unit) {
            RelativeTimeUnit.MINUTE -> pluralStringResource(Res.plurals.relative_time_minute, value.amount, value.amount)
            RelativeTimeUnit.HOUR -> pluralStringResource(Res.plurals.relative_time_hour, value.amount, value.amount)
            RelativeTimeUnit.DAY -> pluralStringResource(Res.plurals.relative_time_day, value.amount, value.amount)
            RelativeTimeUnit.WEEK -> pluralStringResource(Res.plurals.relative_time_week, value.amount, value.amount)
            RelativeTimeUnit.MONTH -> pluralStringResource(Res.plurals.relative_time_month, value.amount, value.amount)
            RelativeTimeUnit.YEAR -> pluralStringResource(Res.plurals.relative_time_year, value.amount, value.amount)
            RelativeTimeUnit.NOW -> error("handled above")
        }

    return stringResource(
        if (value.future) Res.string.relative_time_future else Res.string.relative_time_past,
        duration,
    )
}

private const val MINUTE_SECONDS = 60L
private const val HOUR_SECONDS = 60L * MINUTE_SECONDS
private const val DAY_SECONDS = 24L * HOUR_SECONDS
private const val WEEK_SECONDS = 7L * DAY_SECONDS
private const val MONTH_SECONDS = 30L * DAY_SECONDS
private const val YEAR_SECONDS = 365L * DAY_SECONDS
