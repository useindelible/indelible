package app.indelible.ui.components

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextField
import androidx.compose.material3.TextFieldDefaults
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.text.input.VisualTransformation
import app.indelible.ui.theme.IndelibleSpacing

/**
 * The canonical text input for Indelible.
 *
 * Uses the filled TextField variant with:
 *   - containerColor: surfaceVariant → bg-secondary (#F5F5F7 / #1C1C1E)
 *   - focusedIndicatorColor / unfocusedIndicatorColor: hidden (no underline)
 *   - focusedLabelColor:   primary → accent
 *   - cursorColor:         primary → accent
 *   - Shape:               MaterialTheme.shapes.small → radius-sm (7dp)
 *
 * Use this wherever a text input appears. Do NOT create local TextField
 * composables in individual screens.
 */
@Composable
fun IndelibleTextField(
    value: String,
    onValueChange: (String) -> Unit,
    label: String,
    modifier: Modifier = Modifier,
    error: String? = null,
    enabled: Boolean = true,
    isPassword: Boolean = false,
    keyboardType: KeyboardType = KeyboardType.Text,
    imeAction: ImeAction = ImeAction.Next,
    onImeAction: () -> Unit = {},
    singleLine: Boolean = true,
    minLines: Int = 1,
    maxLines: Int = if (singleLine) 1 else Int.MAX_VALUE,
) {
    Column(modifier = modifier.fillMaxWidth()) {
        TextField(
            value = value,
            onValueChange = onValueChange,
            label = {
                Text(
                    text = label,
                    style = MaterialTheme.typography.bodyMedium, // subheadline
                )
            },
            enabled = enabled,
            isError = error != null,
            visualTransformation =
                if (isPassword) {
                    PasswordVisualTransformation()
                } else {
                    VisualTransformation.None
                },
            keyboardOptions =
                KeyboardOptions(
                    keyboardType = keyboardType,
                    imeAction = imeAction,
                ),
            keyboardActions =
                KeyboardActions(
                    onDone = { onImeAction() },
                    onNext = { onImeAction() },
                ),
            singleLine = singleLine,
            minLines = minLines,
            maxLines = maxLines,
            colors =
                TextFieldDefaults.colors(
                    // Container: bg-secondary — filled style per prototype
                    focusedContainerColor = MaterialTheme.colorScheme.surfaceVariant,
                    unfocusedContainerColor = MaterialTheme.colorScheme.surfaceVariant,
                    errorContainerColor = MaterialTheme.colorScheme.surfaceVariant,
                    // Remove the bottom divider line (filled fields don't need it)
                    focusedIndicatorColor = Color.Transparent,
                    unfocusedIndicatorColor = Color.Transparent,
                    errorIndicatorColor = MaterialTheme.colorScheme.error,
                    // Accent-coloured label when focused
                    focusedLabelColor = MaterialTheme.colorScheme.primary,
                    unfocusedLabelColor = MaterialTheme.colorScheme.onSurfaceVariant,
                    // Cursor
                    cursorColor = MaterialTheme.colorScheme.primary,
                    // Error label
                    errorLabelColor = MaterialTheme.colorScheme.error,
                    errorCursorColor = MaterialTheme.colorScheme.error,
                ),
            modifier = Modifier.fillMaxWidth(),
        )
        if (error != null) {
            Text(
                text = error,
                color = MaterialTheme.colorScheme.error,
                style = MaterialTheme.typography.bodySmall, // footnote: 12sp
                modifier =
                    Modifier.padding(
                        start = IndelibleSpacing.step16,
                        top = IndelibleSpacing.step4,
                    ),
            )
        }
    }
}
