package app.indelible.ui.components

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.size
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import app.indelible.ui.theme.IndelibleSpacing

enum class IndelibleButtonStyle { Primary, Secondary, Destructive, OutlinedDestructive, Text }

/**
 * The canonical action button for Indelible.
 *
 * Styling is driven entirely by [AppTheme]:
 *   - Primary:     Background = primary (accent), label = onPrimary (white). Full-width + touchTarget unless compact.
 *   - Secondary:   Outlined border, surface background, auto-width.
 *   - Destructive: Background = error, label = onError. Full-width + touchTarget unless compact.
 *   - Text:        Text-only, no background.
 *   - compact:     Removes forced full-width and uses step40 height instead of touchTarget.
 *
 * Use this everywhere a button is needed.
 * Do NOT create local Button composables in individual screens.
 */
@Composable
fun IndelibleButton(
    text: String,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    isLoading: Boolean = false,
    enabled: Boolean = true,
    style: IndelibleButtonStyle = IndelibleButtonStyle.Primary,
    compact: Boolean = false,
) {
    val height = if (compact) IndelibleSpacing.step40 else IndelibleSpacing.touchTarget

    when (style) {
        IndelibleButtonStyle.Text -> {
            TextButton(
                onClick = onClick,
                enabled = enabled && !isLoading,
                modifier = modifier.height(height),
            ) {
                Text(
                    text = text,
                    style = MaterialTheme.typography.titleSmall,
                    color = MaterialTheme.colorScheme.primary,
                )
            }
        }

        IndelibleButtonStyle.Secondary -> {
            OutlinedButton(
                onClick = onClick,
                enabled = enabled && !isLoading,
                colors =
                    ButtonDefaults.outlinedButtonColors(
                        contentColor = MaterialTheme.colorScheme.onSurface,
                    ),
                border = BorderStroke(0.5.dp, MaterialTheme.colorScheme.outline),
                modifier = modifier.height(height),
            ) {
                Text(
                    text = text,
                    style = MaterialTheme.typography.titleSmall,
                )
            }
        }

        IndelibleButtonStyle.OutlinedDestructive -> {
            OutlinedButton(
                onClick = onClick,
                enabled = enabled && !isLoading,
                colors =
                    ButtonDefaults.outlinedButtonColors(
                        contentColor = MaterialTheme.colorScheme.error,
                    ),
                border = BorderStroke(0.5.dp, MaterialTheme.colorScheme.error),
                modifier = modifier.height(height),
            ) {
                Text(
                    text = text,
                    style = MaterialTheme.typography.titleSmall,
                )
            }
        }

        else -> {
            val colors =
                when (style) {
                    IndelibleButtonStyle.Destructive ->
                        ButtonDefaults.buttonColors(
                            containerColor = MaterialTheme.colorScheme.error,
                            contentColor = MaterialTheme.colorScheme.onError,
                        )
                    else -> ButtonDefaults.buttonColors()
                }
            val sizeModifier =
                if (compact) {
                    modifier.height(height)
                } else {
                    modifier.fillMaxWidth().height(IndelibleSpacing.touchTarget)
                }
            Button(
                onClick = onClick,
                enabled = enabled && !isLoading,
                colors = colors,
                modifier = sizeModifier,
            ) {
                if (isLoading) {
                    CircularProgressIndicator(
                        modifier = Modifier.size(IndelibleSpacing.step24),
                        color = MaterialTheme.colorScheme.onPrimary,
                        strokeWidth = IndelibleSpacing.step2,
                    )
                } else {
                    Text(
                        text = text,
                        style = MaterialTheme.typography.titleSmall,
                    )
                }
            }
        }
    }
}
