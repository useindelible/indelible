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
import app.indelible.ui.theme.IndelibleSpacing

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
            text = "Indelible",
            style = MaterialTheme.typography.headlineMedium,
            textAlign = TextAlign.Center,
            modifier = Modifier.align(Alignment.CenterHorizontally),
        )

        if (signupsEnabled) {
            Spacer(modifier = Modifier.height(IndelibleSpacing.step8))

            Text(
                text = if (setupRequired) "Create the first account to get started" else "Create your account",
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                textAlign = TextAlign.Center,
                modifier = Modifier.align(Alignment.CenterHorizontally),
            )

            Spacer(modifier = Modifier.height(IndelibleSpacing.step24))

            if (registerState.serverError != null) {
                Text(
                    text = registerState.serverError.orEmpty(),
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
                label = "Display Name",
                error = registerState.displayNameError,
            )

            Spacer(modifier = Modifier.height(IndelibleSpacing.step12))

            AuthTextField(
                value = registerState.email,
                onValueChange = viewModel::updateRegisterEmail,
                label = "Email",
                error = registerState.emailError,
                keyboardType = KeyboardType.Email,
            )

            Spacer(modifier = Modifier.height(IndelibleSpacing.step12))

            AuthTextField(
                value = registerState.password,
                onValueChange = viewModel::updateRegisterPassword,
                label = "Password",
                error = registerState.passwordError,
                isPassword = true,
            )

            Spacer(modifier = Modifier.height(IndelibleSpacing.step12))

            AuthTextField(
                value = registerState.confirmPassword,
                onValueChange = viewModel::updateRegisterConfirmPassword,
                label = "Confirm Password",
                error = registerState.confirmPasswordError,
                isPassword = true,
                imeAction = ImeAction.Done,
                onImeAction = { viewModel.register() },
            )

            Spacer(modifier = Modifier.height(IndelibleSpacing.step24))

            AuthButton(
                text = "Create Account",
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
            Text("Already have an account? Sign in")
        }
    }
}
