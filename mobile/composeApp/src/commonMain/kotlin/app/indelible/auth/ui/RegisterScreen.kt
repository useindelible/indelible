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
import app.indelible.auth.ui.components.AuthButton
import app.indelible.auth.ui.components.AuthCard
import app.indelible.auth.ui.components.AuthTextField
import app.indelible.auth.ui.components.OAuthButton
import app.indelible.auth.viewmodel.AuthViewModel
import app.indelible.core.i18n.resolve
import app.indelible.ui.theme.IndelibleSpacing
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.auth_already_have_account
import indelible.composeapp.generated.resources.auth_confirm_password_label
import indelible.composeapp.generated.resources.auth_create_account
import indelible.composeapp.generated.resources.auth_create_first_account
import indelible.composeapp.generated.resources.auth_create_your_account
import indelible.composeapp.generated.resources.auth_display_name_label
import indelible.composeapp.generated.resources.auth_email_label
import indelible.composeapp.generated.resources.auth_password_label
import indelible.composeapp.generated.resources.common_app_name
import org.jetbrains.compose.resources.stringResource

@Composable
fun RegisterScreen(
    viewModel: AuthViewModel,
    onNavigateToLogin: () -> Unit,
) {
    val registerState by viewModel.registerState.collectAsState()
    val oauthProviders by viewModel.oauthProviders.collectAsState()
    val signupsEnabled by viewModel.signupsEnabled.collectAsState()
    val setupRequired by viewModel.setupRequired.collectAsState()

    AuthCard {
        Text(
            text = stringResource(Res.string.common_app_name),
            style = MaterialTheme.typography.headlineMedium,
            textAlign = TextAlign.Center,
            modifier = Modifier.align(Alignment.CenterHorizontally),
        )

        if (signupsEnabled) {
            Spacer(modifier = Modifier.height(IndelibleSpacing.step8))

            Text(
                text =
                    stringResource(
                        if (setupRequired) {
                            Res.string.auth_create_first_account
                        } else {
                            Res.string.auth_create_your_account
                        },
                    ),
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                textAlign = TextAlign.Center,
                modifier = Modifier.align(Alignment.CenterHorizontally),
            )

            Spacer(modifier = Modifier.height(IndelibleSpacing.step24))

            if (registerState.serverError != null) {
                Text(
                    text = registerState.serverError?.resolve().orEmpty(),
                    color = MaterialTheme.colorScheme.error,
                    style = MaterialTheme.typography.bodySmall,
                    textAlign = TextAlign.Center,
                    modifier = Modifier.align(Alignment.CenterHorizontally),
                )
                Spacer(modifier = Modifier.height(IndelibleSpacing.step8))
            }

            AuthTextField(
                value = registerState.displayName,
                onValueChange = viewModel::updateRegisterDisplayName,
                label = stringResource(Res.string.auth_display_name_label),
                error = registerState.displayNameError?.resolve(),
            )

            Spacer(modifier = Modifier.height(IndelibleSpacing.step12))

            AuthTextField(
                value = registerState.email,
                onValueChange = viewModel::updateRegisterEmail,
                label = stringResource(Res.string.auth_email_label),
                error = registerState.emailError?.resolve(),
                keyboardType = KeyboardType.Email,
            )

            Spacer(modifier = Modifier.height(IndelibleSpacing.step12))

            AuthTextField(
                value = registerState.password,
                onValueChange = viewModel::updateRegisterPassword,
                label = stringResource(Res.string.auth_password_label),
                error = registerState.passwordError?.resolve(),
                isPassword = true,
            )

            Spacer(modifier = Modifier.height(IndelibleSpacing.step12))

            AuthTextField(
                value = registerState.confirmPassword,
                onValueChange = viewModel::updateRegisterConfirmPassword,
                label = stringResource(Res.string.auth_confirm_password_label),
                error = registerState.confirmPasswordError?.resolve(),
                isPassword = true,
                imeAction = ImeAction.Done,
                onImeAction = { viewModel.register() },
            )

            Spacer(modifier = Modifier.height(IndelibleSpacing.step24))

            AuthButton(
                text = stringResource(Res.string.auth_create_account),
                onClick = { viewModel.register() },
                isLoading = registerState.isLoading,
            )

            Spacer(modifier = Modifier.height(IndelibleSpacing.step16))

            oauthProviders.forEach { provider ->
                OAuthButton(
                    providerName = provider.name,
                    onClick = { viewModel.startOAuth(provider.id) },
                )
                Spacer(modifier = Modifier.height(IndelibleSpacing.step8))
            }
        }

        TextButton(
            onClick = onNavigateToLogin,
            modifier = Modifier.align(Alignment.CenterHorizontally),
        ) {
            Text(stringResource(Res.string.auth_already_have_account))
        }
    }
}
