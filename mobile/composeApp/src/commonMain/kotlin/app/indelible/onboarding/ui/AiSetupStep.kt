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
        title = "AI Assistant",
        subtitle = "Set up Mila, your AI reading companion",
        modifier = modifier,
    ) {
        AiProvider.entries.filter { it != AiProvider.NONE }.forEach { provider ->
            ProviderCard(
                title = provider.label,
                description = aiProviderDescription(provider),
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
                label = "Ollama Endpoint URL",
            )
        } else if (selectedProvider != AiProvider.NONE) {
            Spacer(modifier = Modifier.height(IndelibleSpacing.step8))
            IndelibleTextField(
                value = apiKeyInput,
                onValueChange = onApiKeyChange,
                label = "API Key",
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
                text =
                    "Your API keys are stored securely and never shared. " +
                        "Mila uses them to summarize, tag, and help you find your saved content.",
                style = MaterialTheme.typography.bodySmall, // footnote: 12sp/400
                color = MaterialTheme.colorScheme.onPrimaryContainer,
                modifier =
                    Modifier
                        .fillMaxWidth()
                        .padding(IndelibleSpacing.step16),
            )
        }

        Spacer(modifier = Modifier.height(IndelibleSpacing.step32))

        IndelibleButton(text = "Continue", onClick = onContinue)

        Spacer(modifier = Modifier.height(IndelibleSpacing.step8))

        TextButton(
            onClick = onSkip,
            modifier = Modifier.fillMaxWidth(),
        ) {
            Text(
                text = "Skip",
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
    }
}

private fun aiProviderDescription(provider: AiProvider): String =
    when (provider) {
        AiProvider.NONE -> ""
        AiProvider.OLLAMA -> "Run AI models locally on your own hardware"
        AiProvider.OPENAI -> "Use OpenAI-compatible providers, including OpenRouter for Claude"
    }
