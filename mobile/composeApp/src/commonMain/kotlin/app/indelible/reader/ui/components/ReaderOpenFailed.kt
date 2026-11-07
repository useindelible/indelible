package app.indelible.reader.ui.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextAlign
import app.indelible.ui.components.IndelibleButton
import app.indelible.ui.components.IndelibleButtonStyle
import app.indelible.ui.theme.IndelibleSpacing
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.common_back
import indelible.composeapp.generated.resources.common_retry
import org.jetbrains.compose.resources.stringResource

/** The reader has no chrome before an item opens, so this screen carries its own way back. */
@Composable
fun ReaderOpenFailed(
    message: String,
    onRetry: () -> Unit,
    onBack: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Column(
        modifier = modifier.fillMaxSize().padding(horizontal = IndelibleSpacing.screenPaddingH),
        verticalArrangement = Arrangement.spacedBy(IndelibleSpacing.step12, Alignment.CenterVertically),
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        Text(
            text = message,
            style = MaterialTheme.typography.bodyLarge,
            color = MaterialTheme.colorScheme.error,
            textAlign = TextAlign.Center,
        )
        IndelibleButton(text = stringResource(Res.string.common_retry), onClick = onRetry, compact = true)
        IndelibleButton(
            text = stringResource(Res.string.common_back),
            onClick = onBack,
            style = IndelibleButtonStyle.Text,
            compact = true,
        )
    }
}
