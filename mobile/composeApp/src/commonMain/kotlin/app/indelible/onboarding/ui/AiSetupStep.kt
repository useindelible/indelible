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
import app.indelible.onboarding.ui.components.ProviderCard
import app.indelible.onboarding.ui.components.StepCard
import app.indelible.onboarding.viewmodel.AiProvider
import app.indelible.ui.components.IndelibleButton
import app.indelible.ui.components.IndelibleTextField
import app.indelible.ui.theme.IndelibleSpacing
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.common_continue
import indelible.composeapp.generated.resources.onboarding_ai_api_key_label
import indelible.composeapp.generated.resources.onboarding_ai_ollama_description
import indelible.composeapp.generated.resources.onboarding_ai_ollama_endpoint_label
import indelible.composeapp.generated.resources.onboarding_ai_openai_description
import indelible.composeapp.generated.resources.onboarding_ai_privacy
import indelible.composeapp.generated.resources.onboarding_ai_subtitle
import indelible.composeapp.generated.resources.onboarding_ai_title
import indelible.composeapp.generated.resources.onboarding_skip
import org.jetbrains.compose.resources.StringResource
import org.jetbrains.compose.resources.stringResource

@Composable
fun AiSetupStep(
    selectedProvider: AiProvider,
    apiKeyInput: String,
    onSelectProvider: (AiProvider) -> Unit,
    onApiKeyChange: (String) -> Unit,
    onContinue: () -> Unit,
    onSkip: () -> Unit,
    modifier: Modifier = Modifier,
) {
    StepCard(
        title = stringResource(Res.string.onboarding_ai_title),
        subtitle = stringResource(Res.string.onboarding_ai_subtitle),
        modifier = modifier,
    ) {
        AiProvider.entries.filter { it != AiProvider.NONE }.forEach { provider ->
            ProviderCard(
                title = stringResource(provider.labelRes),
                description = stringResource(aiProviderDescriptionRes(provider)),
                isSelected = selectedProvider == provider,
                onClick = { onSelectProvider(provider) },
            )
            Spacer(modifier = Modifier.height(IndelibleSpacing.step8))
        }

        if (selectedProvider == AiProvider.OLLAMA) {
            Spacer(modifier = Modifier.height(IndelibleSpacing.step8))
            IndelibleTextField(
                value = apiKeyInput,
                onValueChange = onApiKeyChange,
                label = stringResource(Res.string.onboarding_ai_ollama_endpoint_label),
            )
        } else if (selectedProvider != AiProvider.NONE) {
            Spacer(modifier = Modifier.height(IndelibleSpacing.step8))
            IndelibleTextField(
                value = apiKeyInput,
                onValueChange = onApiKeyChange,
                label = stringResource(Res.string.onboarding_ai_api_key_label),
                isPassword = true,
            )
        }

        Spacer(modifier = Modifier.height(IndelibleSpacing.step16))

        Card(
            colors =
                CardDefaults.cardColors(
                    containerColor = MaterialTheme.colorScheme.primaryContainer,
                ),
            shape = MaterialTheme.shapes.medium,
            modifier = Modifier.fillMaxWidth(),
        ) {
            Text(
                text = stringResource(Res.string.onboarding_ai_privacy),
                style = MaterialTheme.typography.bodySmall, // footnote: 12sp/400
                color = MaterialTheme.colorScheme.onPrimaryContainer,
                modifier =
                    Modifier
                        .fillMaxWidth()
                        .padding(IndelibleSpacing.step16),
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

private fun aiProviderDescriptionRes(provider: AiProvider): StringResource =
    when (provider) {
        AiProvider.NONE -> error("NONE has no onboarding description")
        AiProvider.OLLAMA -> Res.string.onboarding_ai_ollama_description
        AiProvider.OPENAI -> Res.string.onboarding_ai_openai_description
    }
