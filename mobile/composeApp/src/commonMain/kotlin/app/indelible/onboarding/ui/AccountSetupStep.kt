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
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.common_continue
import indelible.composeapp.generated.resources.onboarding_account_display_name
import indelible.composeapp.generated.resources.onboarding_account_subtitle
import indelible.composeapp.generated.resources.onboarding_account_theme
import indelible.composeapp.generated.resources.onboarding_account_title
import indelible.composeapp.generated.resources.onboarding_skip
import indelible.composeapp.generated.resources.onboarding_theme_auto
import indelible.composeapp.generated.resources.onboarding_theme_dark
import indelible.composeapp.generated.resources.onboarding_theme_light
import org.jetbrains.compose.resources.stringResource

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
        title = stringResource(Res.string.onboarding_account_title),
        subtitle = stringResource(Res.string.onboarding_account_subtitle),
        modifier = modifier,
    ) {
        IndelibleTextField(
            value = displayName,
            onValueChange = viewModel::updateDisplayName,
            label = stringResource(Res.string.onboarding_account_display_name),
        )

        Spacer(modifier = Modifier.height(IndelibleSpacing.sectionGap))

        Text(
            text = stringResource(Res.string.onboarding_account_theme),
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
                                ThemeChoice.LIGHT -> stringResource(Res.string.onboarding_theme_light)
                                ThemeChoice.DARK -> stringResource(Res.string.onboarding_theme_dark)
                                ThemeChoice.AUTO -> stringResource(Res.string.onboarding_theme_auto)
                            },
                            style = MaterialTheme.typography.bodyMedium, // subheadline
                        )
                    },
                )
            }
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
                style = MaterialTheme.typography.bodyMedium, // subheadline
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
    }
}
