package app.indelible.auth.ui

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
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
fun LoginScreen(
    viewModel: AuthViewModel,
    onNavigateToRegister: () -> Unit,
    onNavigateToForgotPassword: () -> Unit,
    serverHost: String? = null,
    onChangeServer: (() -> Unit)? = null,
) {
    val loginState by viewModel.loginState.collectAsState()
    val oauthProviders by viewModel.oauthProviders.collectAsState()
    val signupsEnabled by viewModel.signupsEnabled.collectAsState()

    AuthCard {
        Text(
            text = "Indelible",
            style = MaterialTheme.typography.headlineMedium,
            textAlign = TextAlign.Center,
            modifier = Modifier.align(Alignment.CenterHorizontally),
        )

        Spacer(modifier = Modifier.height(IndelibleSpacing.step8))

        Text(
            text = "Sign in to your account",
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            textAlign = TextAlign.Center,
            modifier = Modifier.align(Alignment.CenterHorizontally),
        )

        if (serverHost != null && onChangeServer != null) {
            Row(
                modifier = Modifier.align(Alignment.CenterHorizontally),
                horizontalArrangement = Arrangement.Center,
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Text(
                    text = serverHost,
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
                TextButton(onClick = onChangeServer) {
                    Text("Change")
                }
            }
        }

        Spacer(modifier = Modifier.height(IndelibleSpacing.step24))

        if (loginState.serverError != null) {
            Text(
                text = loginState.serverError.orEmpty(),
                color = MaterialTheme.colorScheme.error,
                style = MaterialTheme.typography.bodySmall,
                textAlign = TextAlign.Center,
                modifier = Modifier.align(Alignment.CenterHorizontally),
            )
            Spacer(modifier = Modifier.height(IndelibleSpacing.step8))
        }

        AuthTextField(
            value = loginState.email,
            onValueChange = viewModel::updateLoginEmail,
            label = "Email",
            error = loginState.emailError,
            keyboardType = KeyboardType.Email,
        )

        Spacer(modifier = Modifier.height(IndelibleSpacing.step12))

        AuthTextField(
            value = loginState.password,
            onValueChange = viewModel::updateLoginPassword,
            label = "Password",
            error = loginState.passwordError,
            isPassword = true,
            imeAction = ImeAction.Done,
            onImeAction = { viewModel.login() },
        )

        Spacer(modifier = Modifier.height(IndelibleSpacing.step8))

        TextButton(
            onClick = onNavigateToForgotPassword,
            modifier = Modifier.align(Alignment.End),
        ) {
            Text("Forgot password?")
        }

        Spacer(modifier = Modifier.height(IndelibleSpacing.step8))

        AuthButton(
            text = "Sign In",
            onClick = { viewModel.login() },
            isLoading = loginState.isLoading,
        )

        Spacer(modifier = Modifier.height(IndelibleSpacing.step16))

        oauthProviders.forEach { provider ->
            OAuthButton(
                providerName = provider.name,
                onClick = { viewModel.startOAuth(provider.id) },
            )
            Spacer(modifier = Modifier.height(IndelibleSpacing.step8))
        }

        if (signupsEnabled) {
            TextButton(
                onClick = onNavigateToRegister,
                modifier = Modifier.align(Alignment.CenterHorizontally),
            ) {
                Text("Don't have an account? Register")
            }
        }
    }
}
