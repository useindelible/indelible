package app.indelible.auth.ui.components

import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardType
import app.indelible.ui.components.IndelibleButton
import app.indelible.ui.components.IndelibleTextField

/**
 * Auth-specific button — delegates to [IndelibleButton].
 * Kept as a thin wrapper so auth screens don't need to import from ui.components directly.
 */
@Composable
fun AuthButton(
    text: String,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    isLoading: Boolean = false,
    enabled: Boolean = true,
) {
    IndelibleButton(
        text = text,
        onClick = onClick,
        modifier = modifier,
        isLoading = isLoading,
        enabled = enabled,
    )
}

/**
 * Auth-specific text field — delegates to [IndelibleTextField].
 * Kept as a thin wrapper so auth screens don't need to import from ui.components directly.
 */
@Composable
fun AuthTextField(
    value: String,
    onValueChange: (String) -> Unit,
    label: String,
    modifier: Modifier = Modifier,
    error: String? = null,
    isPassword: Boolean = false,
    keyboardType: KeyboardType = KeyboardType.Text,
    imeAction: ImeAction = ImeAction.Next,
    onImeAction: () -> Unit = {},
) {
    IndelibleTextField(
        value = value,
        onValueChange = onValueChange,
        label = label,
        modifier = modifier,
        error = error,
        isPassword = isPassword,
        keyboardType = keyboardType,
        imeAction = imeAction,
        onImeAction = onImeAction,
    )
}
