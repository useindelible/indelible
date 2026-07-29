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
import app.indelible.home.viewmodel.Greeting
import app.indelible.ui.theme.AppTheme
import app.indelible.ui.theme.IndelibleSpacing
import kotlinx.datetime.Clock
import kotlinx.datetime.TimeZone
import kotlinx.datetime.todayIn

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

private fun greetingText(greeting: Greeting): String =
    when (greeting) {
        Greeting.MORNING -> "Good morning"
        Greeting.AFTERNOON -> "Good afternoon"
        Greeting.EVENING -> "Good evening"
    }

private fun todayLabel(): String {
    val date = Clock.System.todayIn(TimeZone.currentSystemDefault())
    val day = abbreviate(date.dayOfWeek.name)
    val month = abbreviate(date.month.name)
    return "$day · ${date.dayOfMonth} $month"
}

private const val ABBREVIATION_LENGTH = 3

private fun abbreviate(enumName: String): String =
    enumName
        .lowercase()
        .replaceFirstChar { it.uppercase() }
        .take(ABBREVIATION_LENGTH)

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
