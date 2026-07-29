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
fun ReadyStep(
    onComplete: () -> Unit,
    modifier: Modifier = Modifier,
) {
    StepCard(
        title = "You're All Set!",
        subtitle = "Your Indelible library is ready",
        modifier = modifier,
    ) {
        Text(
            text =
                "Start saving articles, documents, and links. " +
                    "Everything you save is archived, searchable, and always available.",
            style = MaterialTheme.typography.bodyLarge,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            textAlign = TextAlign.Center,
            modifier = Modifier.fillMaxWidth(),
        )

        Spacer(modifier = Modifier.height(IndelibleSpacing.step48))

        IndelibleButton(text = "Go to Library", onClick = onComplete)
    }
}
