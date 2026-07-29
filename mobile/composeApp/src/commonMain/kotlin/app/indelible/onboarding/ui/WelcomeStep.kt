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

@Composable
fun WelcomeStep(
    onContinue: () -> Unit,
    modifier: Modifier = Modifier,
) {
    StepCard(
        title = "Welcome to Indelible",
        subtitle = "Your personal read-it-later and knowledge archive",
        modifier = modifier,
    ) {
        Text(
            text =
                "Save articles, documents, and links from anywhere. " +
                    "Indelible keeps your reading organized and searchable, " +
                    "so nothing important slips through the cracks.",
            style = MaterialTheme.typography.bodyLarge,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            textAlign = TextAlign.Center,
            modifier = Modifier.fillMaxWidth(),
        )

        Spacer(modifier = Modifier.height(IndelibleSpacing.step16))

        Text(
            text = "Let's get you set up in just a few steps.",
            style = MaterialTheme.typography.bodyLarge, // body: 15sp/400
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            textAlign = TextAlign.Center,
            modifier = Modifier.fillMaxWidth(),
        )

        Spacer(modifier = Modifier.height(IndelibleSpacing.step48))

        IndelibleButton(text = "Get Started", onClick = onContinue)
    }
}
