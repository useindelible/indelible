package app.indelible.onboarding.ui

import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextAlign
import app.indelible.onboarding.ui.components.StepCard
import app.indelible.ui.components.IndelibleButton
import app.indelible.ui.theme.IndelibleSpacing
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.onboarding_get_started
import indelible.composeapp.generated.resources.onboarding_welcome_body
import indelible.composeapp.generated.resources.onboarding_welcome_setup
import indelible.composeapp.generated.resources.onboarding_welcome_subtitle
import indelible.composeapp.generated.resources.onboarding_welcome_title
import org.jetbrains.compose.resources.stringResource

@Composable
fun WelcomeStep(
    onContinue: () -> Unit,
    modifier: Modifier = Modifier,
) {
    StepCard(
        title = stringResource(Res.string.onboarding_welcome_title),
        subtitle = stringResource(Res.string.onboarding_welcome_subtitle),
        modifier = modifier,
    ) {
        Text(
            text = stringResource(Res.string.onboarding_welcome_body),
            style = MaterialTheme.typography.bodyLarge,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            textAlign = TextAlign.Center,
            modifier = Modifier.fillMaxWidth(),
        )

        Spacer(modifier = Modifier.height(IndelibleSpacing.step16))

        Text(
            text = stringResource(Res.string.onboarding_welcome_setup),
            style = MaterialTheme.typography.bodyLarge, // body: 15sp/400
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            textAlign = TextAlign.Center,
            modifier = Modifier.fillMaxWidth(),
        )

        Spacer(modifier = Modifier.height(IndelibleSpacing.step48))

        IndelibleButton(text = stringResource(Res.string.onboarding_get_started), onClick = onContinue)
    }
}
