package app.indelible.onboarding.ui

import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import app.indelible.onboarding.ui.components.StepCard
import app.indelible.ui.components.IndelibleButton
import app.indelible.ui.components.IndelibleTextField
import app.indelible.ui.theme.IndelibleSpacing
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.common_continue
import indelible.composeapp.generated.resources.onboarding_add_content_email_tip
import indelible.composeapp.generated.resources.onboarding_add_content_share_tip
import indelible.composeapp.generated.resources.onboarding_add_content_subtitle
import indelible.composeapp.generated.resources.onboarding_add_content_title
import indelible.composeapp.generated.resources.onboarding_add_content_url_label
import indelible.composeapp.generated.resources.onboarding_skip
import org.jetbrains.compose.resources.stringResource

@Composable
fun AddContentStep(
    urlInput: String,
    onUrlChange: (String) -> Unit,
    onContinue: () -> Unit,
    onSkip: () -> Unit,
    modifier: Modifier = Modifier,
) {
    StepCard(
        title = stringResource(Res.string.onboarding_add_content_title),
        subtitle = stringResource(Res.string.onboarding_add_content_subtitle),
        modifier = modifier,
    ) {
        IndelibleTextField(
            value = urlInput,
            onValueChange = onUrlChange,
            label = stringResource(Res.string.onboarding_add_content_url_label),
        )

        Spacer(modifier = Modifier.height(IndelibleSpacing.sectionGap))

        // Info card — uses primaryContainer (fill-selected) as a subtle tip background
        Card(
            colors =
                CardDefaults.cardColors(
                    containerColor = MaterialTheme.colorScheme.primaryContainer,
                ),
            shape = MaterialTheme.shapes.medium,
            modifier = Modifier.fillMaxWidth(),
        ) {
            Text(
                text = stringResource(Res.string.onboarding_add_content_share_tip),
                style = MaterialTheme.typography.bodyMedium, // subheadline: 13sp/400
                color = MaterialTheme.colorScheme.onPrimaryContainer,
                modifier = Modifier.padding(IndelibleSpacing.step16),
            )
        }

        Spacer(modifier = Modifier.height(IndelibleSpacing.step16))

        Card(
            colors =
                CardDefaults.cardColors(
                    containerColor = MaterialTheme.colorScheme.surfaceVariant,
                ),
            shape = MaterialTheme.shapes.medium,
            modifier = Modifier.fillMaxWidth(),
        ) {
            Text(
                text = stringResource(Res.string.onboarding_add_content_email_tip),
                style = MaterialTheme.typography.bodyMedium, // subheadline: 13sp/400
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                modifier = Modifier.padding(IndelibleSpacing.step16),
            )
        }

        Spacer(modifier = Modifier.height(IndelibleSpacing.step32))

        IndelibleButton(text = stringResource(Res.string.common_continue), onClick = onContinue)

        Spacer(modifier = Modifier.height(IndelibleSpacing.step8))

        TextButton(
            onClick = onSkip,
            modifier = Modifier.fillMaxWidth(),
        ) {
            Text(
                text = stringResource(Res.string.onboarding_skip),
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
    }
}
