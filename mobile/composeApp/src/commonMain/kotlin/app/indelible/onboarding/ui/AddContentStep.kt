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

@Composable
fun AddContentStep(
    urlInput: String,
    onUrlChange: (String) -> Unit,
    onContinue: () -> Unit,
    onSkip: () -> Unit,
    modifier: Modifier = Modifier,
) {
    StepCard(
        title = "Add Your First Content",
        subtitle = "Save a link to get started",
        modifier = modifier,
    ) {
        IndelibleTextField(
            value = urlInput,
            onValueChange = onUrlChange,
            label = "Paste a URL",
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
                text =
                    "You can also share any link to Indelible from your browser or any app " +
                        "using the system share sheet.",
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
                text = "Forward emails to your personal ingest address to save them automatically.",
                style = MaterialTheme.typography.bodyMedium, // subheadline: 13sp/400
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                modifier = Modifier.padding(IndelibleSpacing.step16),
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
