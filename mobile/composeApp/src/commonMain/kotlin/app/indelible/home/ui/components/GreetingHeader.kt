package app.indelible.home.ui.components

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.tooling.preview.Preview
import app.indelible.core.i18n.LocaleFormatters
import app.indelible.core.i18n.LocalizedDateStyle
import app.indelible.home.viewmodel.Greeting
import app.indelible.ui.theme.AppTheme
import app.indelible.ui.theme.IndelibleSpacing
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.home_greeting_afternoon
import indelible.composeapp.generated.resources.home_greeting_evening
import indelible.composeapp.generated.resources.home_greeting_morning
import kotlinx.datetime.Clock
import org.jetbrains.compose.resources.stringResource

/**
 * Top-of-dashboard greeting: a dated eyebrow ("Mon · 24 May") above a
 * time-of-day salutation. When a name is present it is appended in the accent
 * colour so the eye lands on it.
 */
@Composable
fun GreetingHeader(
    greeting: Greeting,
    name: String?,
    modifier: Modifier = Modifier,
) {
    val accent = MaterialTheme.colorScheme.primary
    val title =
        buildAnnotatedString {
            append(greetingText(greeting))
            if (!name.isNullOrBlank()) {
                append(", ")
                withStyle(SpanStyle(color = accent)) { append(name) }
            }
        }
    Column(modifier = modifier.fillMaxWidth()) {
        Text(
            text = todayLabel(),
            style = homeEyebrowStyle(),
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
        Spacer(Modifier.height(IndelibleSpacing.step6))
        Text(
            text = title,
            style = MaterialTheme.typography.headlineMedium,
            color = MaterialTheme.colorScheme.onSurface,
        )
    }
}

@Composable
private fun greetingText(greeting: Greeting): String =
    when (greeting) {
        Greeting.MORNING -> stringResource(Res.string.home_greeting_morning)
        Greeting.AFTERNOON -> stringResource(Res.string.home_greeting_afternoon)
        Greeting.EVENING -> stringResource(Res.string.home_greeting_evening)
    }

private fun todayLabel(): String =
    LocaleFormatters.date(Clock.System.now(), LocalizedDateStyle.WEEKDAY_MONTH_DAY)

@Preview
@Composable
private fun GreetingHeaderPreviewLight() {
    AppTheme(darkTheme = false) {
        Surface {
            GreetingHeader(
                greeting = Greeting.MORNING,
                name = "Maya",
                modifier = Modifier.padding(IndelibleSpacing.step20),
            )
        }
    }
}

@Preview
@Composable
private fun GreetingHeaderPreviewDark() {
    AppTheme(darkTheme = true) {
        Surface {
            GreetingHeader(
                greeting = Greeting.EVENING,
                name = null,
                modifier = Modifier.padding(IndelibleSpacing.step20),
            )
        }
    }
}
