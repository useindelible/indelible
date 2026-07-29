package app.indelible.reader.ui.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import app.indelible.ui.components.IndelibleButton
import app.indelible.ui.components.IndelibleButtonStyle
import app.indelible.ui.theme.AppTheme
import app.indelible.ui.theme.IndelibleSpacing
import app.indelible.reader.viewmodel.ReaderRetryStatus

/**
 * Shown when the readable render has not landed within the reader's poll budget (common when a
 * feed delivery was just prepared). Offers a manual retry rather than spinning indefinitely.
 */
@Composable
fun ReaderPreparingContent(
    onRetry: () -> Unit,
    retryStatus: ReaderRetryStatus = ReaderRetryStatus.IDLE,
    retryAfterSeconds: Long? = null,
    modifier: Modifier = Modifier,
) {
    Box(
        modifier = modifier.padding(IndelibleSpacing.step32),
        contentAlignment = Alignment.Center,
    ) {
        Column(
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.spacedBy(IndelibleSpacing.step12),
        ) {
            Text(
                text = "Still preparing this article",
                style = MaterialTheme.typography.titleLarge,
                color = MaterialTheme.colorScheme.onSurface,
                textAlign = TextAlign.Center,
            )
            Text(
                text =
                    when (retryStatus) {
                        ReaderRetryStatus.QUEUING -> "Queuing another processing attempt."
                        ReaderRetryStatus.QUEUED -> "Reprocessing is queued."
                        ReaderRetryStatus.COOLDOWN ->
                            retryAfterSeconds?.let { "Retry available in $it seconds." }
                                ?: "Reprocessing is already running."
                        ReaderRetryStatus.IDLE ->
                            "The readable version is taking a moment to render. Try again shortly."
                    },
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                textAlign = TextAlign.Center,
            )
            IndelibleButton(
                text =
                    when (retryStatus) {
                        ReaderRetryStatus.QUEUING -> "Queuing"
                        ReaderRetryStatus.QUEUED -> "Queued"
                        ReaderRetryStatus.COOLDOWN -> "Cooling down"
                        ReaderRetryStatus.IDLE -> "Retry"
                    },
                onClick = onRetry,
                enabled = retryStatus == ReaderRetryStatus.IDLE,
                isLoading = retryStatus == ReaderRetryStatus.QUEUING,
                style = IndelibleButtonStyle.Secondary,
                compact = true,
            )
        }
    }
}

@Preview
@Composable
private fun ReaderPreparingContentLightPreview() {
    AppTheme(darkTheme = false) {
        Surface {
            ReaderPreparingContent(onRetry = {}, modifier = Modifier.fillMaxSize())
        }
    }
}

@Preview
@Composable
private fun ReaderPreparingContentDarkPreview() {
    AppTheme(darkTheme = true) {
        Surface {
            ReaderPreparingContent(onRetry = {}, modifier = Modifier.fillMaxSize())
        }
    }
}
