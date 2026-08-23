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
import indelible.composeapp.generated.resources.onboarding_go_to_library
import indelible.composeapp.generated.resources.onboarding_ready_body
import indelible.composeapp.generated.resources.onboarding_ready_subtitle
import indelible.composeapp.generated.resources.onboarding_ready_title
import org.jetbrains.compose.resources.stringResource

@Composable
fun ReadyStep(
    onComplete: () -> Unit,
    modifier: Modifier = Modifier,
) {
    StepCard(
        title = stringResource(Res.string.onboarding_ready_title),
        subtitle = stringResource(Res.string.onboarding_ready_subtitle),
        modifier = modifier,
    ) {
        Text(
            text = stringResource(Res.string.onboarding_ready_body),
            style = MaterialTheme.typography.bodyLarge,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            textAlign = TextAlign.Center,
            modifier = Modifier.fillMaxWidth(),
        )

        Spacer(modifier = Modifier.height(IndelibleSpacing.step48))

        IndelibleButton(text = stringResource(Res.string.onboarding_go_to_library), onClick = onComplete)
    }
}
