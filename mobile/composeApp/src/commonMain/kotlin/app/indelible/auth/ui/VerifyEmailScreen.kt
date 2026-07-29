package app.indelible.auth.ui

import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.height
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import app.indelible.auth.ui.components.AuthButton
import app.indelible.auth.ui.components.AuthCard
import app.indelible.auth.viewmodel.AuthViewModel
import kotlinx.coroutines.delay

private const val RESEND_COOLDOWN_SECONDS = 60
private const val COOLDOWN_TICK_MS = 1000L

@Composable
fun VerifyEmailScreen(
    viewModel: AuthViewModel,
    email: String,
) {
    var resendCooldown by remember { mutableStateOf(0) }

    LaunchedEffect(Unit) {
        viewModel.pollVerificationStatus()
    }

    LaunchedEffect(resendCooldown) {
        if (resendCooldown > 0) {
            delay(COOLDOWN_TICK_MS)
            resendCooldown--
        }
    }

    AuthCard {
        Text(
            text = "Check your email",
            style = MaterialTheme.typography.headlineMedium,
            textAlign = TextAlign.Center,
            modifier = Modifier.align(Alignment.CenterHorizontally),
        )

        Spacer(modifier = Modifier.height(16.dp))

        Text(
            text = "We sent a verification link to:",
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            textAlign = TextAlign.Center,
            modifier = Modifier.align(Alignment.CenterHorizontally),
        )

        Spacer(modifier = Modifier.height(4.dp))

        Text(
            text = email,
            style = MaterialTheme.typography.bodyLarge,
            textAlign = TextAlign.Center,
            modifier = Modifier.align(Alignment.CenterHorizontally),
        )

        Spacer(modifier = Modifier.height(24.dp))

        Text(
            text =
                "Click the link in the email to verify your account. " +
                    "This page will update automatically once verified.",
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            textAlign = TextAlign.Center,
            modifier = Modifier.align(Alignment.CenterHorizontally),
        )

        Spacer(modifier = Modifier.height(24.dp))

        if (resendCooldown > 0) {
            TextButton(
                onClick = { },
                enabled = false,
                modifier = Modifier.align(Alignment.CenterHorizontally),
            ) {
                Text("Resend email ($resendCooldown s)")
            }
        } else {
            AuthButton(
                text = "Resend email",
                onClick = {
                    viewModel.resendVerification { success ->
                        if (success) {
                            resendCooldown = RESEND_COOLDOWN_SECONDS
                        }
                    }
                },
            )
        }
    }
}
