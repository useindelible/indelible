package app.indelible.auth.ui

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
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
import app.indelible.auth.viewmodel.ConnectServerViewModel
import app.indelible.ui.theme.IndelibleSpacing

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ConnectServerScreen(
    viewModel: ConnectServerViewModel,
    onConnected: () -> Unit,
) {
    val state by viewModel.state.collectAsState()
    val connectedUrl by viewModel.connectedUrl.collectAsState()

    LaunchedEffect(connectedUrl) {
        if (connectedUrl != null) {
            viewModel.consumeConnectedEvent()
            onConnected()
        }
    }

    AuthCard {
        Text(
            text = "Connect to your server",
            style = MaterialTheme.typography.headlineMedium,
            textAlign = TextAlign.Center,
            modifier = Modifier.align(Alignment.CenterHorizontally),
        )

        Spacer(modifier = Modifier.height(IndelibleSpacing.step8))

        Text(
            text = "Enter the address of your Indelible server. It's the same URL you open in your browser.",
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            textAlign = TextAlign.Center,
            modifier = Modifier.align(Alignment.CenterHorizontally),
        )

        Spacer(modifier = Modifier.height(IndelibleSpacing.step24))

        AuthTextField(
            value = state.url,
            onValueChange = viewModel::updateUrl,
            label = "Server address",
            error = state.error,
            keyboardType = KeyboardType.Uri,
            imeAction = ImeAction.Done,
            onImeAction = { viewModel.connect() },
        )

        Spacer(modifier = Modifier.height(IndelibleSpacing.step16))

        AuthButton(
            text = "Continue",
            onClick = { viewModel.connect() },
            isLoading = state.isChecking,
        )
    }

    if (state.pendingCleartextUrl != null) {
        CleartextWarningSheet(
            onContinue = { viewModel.confirmCleartext() },
            onDismiss = { viewModel.dismissCleartextWarning() },
        )
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun CleartextWarningSheet(
    onContinue: () -> Unit,
    onDismiss: () -> Unit,
) {
    val sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true)

    ModalBottomSheet(
        onDismissRequest = onDismiss,
        sheetState = sheetState,
    ) {
        Column(
            modifier =
                Modifier
                    .fillMaxWidth()
                    .padding(
                        horizontal = IndelibleSpacing.screenPaddingH,
                        vertical = IndelibleSpacing.step16,
                    ),
        ) {
            Text(
                text = "This connection isn't encrypted",
                style = MaterialTheme.typography.titleLarge,
                textAlign = TextAlign.Center,
                modifier = Modifier.align(Alignment.CenterHorizontally),
            )

            Spacer(modifier = Modifier.height(IndelibleSpacing.step8))

            Text(
                text =
                    "Traffic to this server, including your password and session, is sent as " +
                        "plain http. Only continue on a network you trust, or put your server " +
                        "behind HTTPS.",
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                textAlign = TextAlign.Center,
                modifier = Modifier.align(Alignment.CenterHorizontally),
            )

            Spacer(modifier = Modifier.height(IndelibleSpacing.step24))

            AuthButton(
                text = "Go back",
                onClick = onDismiss,
            )

            Spacer(modifier = Modifier.height(IndelibleSpacing.step8))

            TextButton(
                onClick = onContinue,
                modifier = Modifier.align(Alignment.CenterHorizontally),
            ) {
                Text(
                    text = "Continue anyway",
                    color = MaterialTheme.colorScheme.error,
                )
            }

            Spacer(modifier = Modifier.height(IndelibleSpacing.step16))
        }
    }
}
