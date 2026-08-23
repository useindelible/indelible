package app.indelible.auth.ui

import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.height
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import app.indelible.auth.ui.components.AuthButton
import app.indelible.auth.ui.components.AuthCard
import app.indelible.auth.ui.components.AuthTextField
import app.indelible.auth.viewmodel.AuthViewModel
import app.indelible.core.i18n.resolve
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.auth_back_to_login
import indelible.composeapp.generated.resources.auth_email_label
import indelible.composeapp.generated.resources.auth_reset_password_body
import indelible.composeapp.generated.resources.auth_reset_password_sent
import indelible.composeapp.generated.resources.auth_reset_password_title
import indelible.composeapp.generated.resources.auth_send_reset_link
import org.jetbrains.compose.resources.stringResource

@Composable
fun ForgotPasswordScreen(
    viewModel: AuthViewModel,
    onNavigateToLogin: () -> Unit,
) {
    val state by viewModel.forgotPasswordState.collectAsState()

    AuthCard {
        Text(
            text = stringResource(Res.string.auth_reset_password_title),
            style = MaterialTheme.typography.headlineMedium,
            textAlign = TextAlign.Center,
            modifier = Modifier.align(Alignment.CenterHorizontally),
        )

        Spacer(modifier = Modifier.height(8.dp))

        if (state.isSubmitted) {
            Text(
                text = stringResource(Res.string.auth_reset_password_sent),
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                textAlign = TextAlign.Center,
                modifier = Modifier.align(Alignment.CenterHorizontally),
            )

            Spacer(modifier = Modifier.height(24.dp))

            TextButton(
                onClick = onNavigateToLogin,
                modifier = Modifier.align(Alignment.CenterHorizontally),
            ) {
                Text(stringResource(Res.string.auth_back_to_login))
            }
        } else {
            Text(
                text = stringResource(Res.string.auth_reset_password_body),
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                textAlign = TextAlign.Center,
                modifier = Modifier.align(Alignment.CenterHorizontally),
            )

            Spacer(modifier = Modifier.height(24.dp))

            if (state.serverError != null) {
                Text(
                    text = state.serverError?.resolve().orEmpty(),
                    color = MaterialTheme.colorScheme.error,
                    style = MaterialTheme.typography.bodySmall,
                    textAlign = TextAlign.Center,
                    modifier = Modifier.align(Alignment.CenterHorizontally),
                )
                Spacer(modifier = Modifier.height(8.dp))
            }

            AuthTextField(
                value = state.email,
                onValueChange = viewModel::updateForgotPasswordEmail,
                label = stringResource(Res.string.auth_email_label),
                error = state.emailError?.resolve(),
                keyboardType = KeyboardType.Email,
                imeAction = ImeAction.Done,
                onImeAction = { viewModel.forgotPassword() },
            )

            Spacer(modifier = Modifier.height(24.dp))

            AuthButton(
                text = stringResource(Res.string.auth_send_reset_link),
                onClick = { viewModel.forgotPassword() },
                isLoading = state.isLoading,
            )

            Spacer(modifier = Modifier.height(16.dp))

            TextButton(
                onClick = onNavigateToLogin,
                modifier = Modifier.align(Alignment.CenterHorizontally),
            ) {
                Text(stringResource(Res.string.auth_back_to_login))
            }
        }
    }
}
