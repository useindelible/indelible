package app.indelible.onboarding.ui

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.material3.FilterChip
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import app.indelible.onboarding.ui.components.StepCard
import app.indelible.onboarding.viewmodel.OnboardingViewModel
import app.indelible.onboarding.viewmodel.ThemeChoice
import app.indelible.ui.components.IndelibleButton
import app.indelible.ui.components.IndelibleTextField
import app.indelible.ui.theme.IndelibleSpacing

@Composable
fun AccountSetupStep(
    viewModel: OnboardingViewModel,
    displayName: String,
    selectedTheme: ThemeChoice,
    onContinue: () -> Unit,
    onSkip: () -> Unit,
    modifier: Modifier = Modifier,
) {
    StepCard(
        title = "Set Up Your Profile",
        subtitle = "Personalize your Indelible experience",
        modifier = modifier,
    ) {
        IndelibleTextField(
            value = displayName,
            onValueChange = viewModel::updateDisplayName,
            label = "Display Name",
        )

        Spacer(modifier = Modifier.height(IndelibleSpacing.sectionGap))

        Text(
            text = "Theme",
            style = MaterialTheme.typography.titleSmall, // callout: 14sp/600
            color = MaterialTheme.colorScheme.onBackground,
        )

        Spacer(modifier = Modifier.height(IndelibleSpacing.step8))

        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step8),
        ) {
            ThemeChoice.entries.forEach { theme ->
                FilterChip(
                    selected = selectedTheme == theme,
                    onClick = { viewModel.updateSelectedTheme(theme) },
                    label = {
                        Text(
                            when (theme) {
                                ThemeChoice.LIGHT -> "Light"
                                ThemeChoice.DARK -> "Dark"
                                ThemeChoice.AUTO -> "Auto"
                            },
                            style = MaterialTheme.typography.bodyMedium, // subheadline
                        )
                    },
                )
            }
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
                style = MaterialTheme.typography.bodyMedium, // subheadline
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
    }
}
