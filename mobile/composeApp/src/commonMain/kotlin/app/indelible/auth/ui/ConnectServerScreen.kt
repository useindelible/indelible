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
import app.indelible.core.i18n.resolve
import app.indelible.ui.theme.IndelibleSpacing
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.auth_cleartext_body
import indelible.composeapp.generated.resources.auth_cleartext_continue
import indelible.composeapp.generated.resources.auth_cleartext_go_back
import indelible.composeapp.generated.resources.auth_cleartext_title
import indelible.composeapp.generated.resources.auth_connect_body
import indelible.composeapp.generated.resources.auth_connect_title
import indelible.composeapp.generated.resources.auth_server_address_label
import indelible.composeapp.generated.resources.common_continue
import org.jetbrains.compose.resources.stringResource

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
            text = stringResource(Res.string.auth_connect_title),
            style = MaterialTheme.typography.headlineMedium,
            textAlign = TextAlign.Center,
            modifier = Modifier.align(Alignment.CenterHorizontally),
        )

        Spacer(modifier = Modifier.height(IndelibleSpacing.step8))

        Text(
            text = stringResource(Res.string.auth_connect_body),
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            textAlign = TextAlign.Center,
            modifier = Modifier.align(Alignment.CenterHorizontally),
        )

        Spacer(modifier = Modifier.height(IndelibleSpacing.step24))

        AuthTextField(
            value = state.url,
            onValueChange = viewModel::updateUrl,
            label = stringResource(Res.string.auth_server_address_label),
            error = state.error?.resolve(),
            keyboardType = KeyboardType.Uri,
            imeAction = ImeAction.Done,
            onImeAction = { viewModel.connect() },
        )

        Spacer(modifier = Modifier.height(IndelibleSpacing.step16))

        AuthButton(
            text = stringResource(Res.string.common_continue),
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
                text = stringResource(Res.string.auth_cleartext_title),
                style = MaterialTheme.typography.titleLarge,
                textAlign = TextAlign.Center,
                modifier = Modifier.align(Alignment.CenterHorizontally),
            )

            Spacer(modifier = Modifier.height(IndelibleSpacing.step8))

            Text(
                text = stringResource(Res.string.auth_cleartext_body),
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                textAlign = TextAlign.Center,
                modifier = Modifier.align(Alignment.CenterHorizontally),
            )

            Spacer(modifier = Modifier.height(IndelibleSpacing.step24))

            AuthButton(
                text = stringResource(Res.string.auth_cleartext_go_back),
                onClick = onDismiss,
            )

            Spacer(modifier = Modifier.height(IndelibleSpacing.step8))

            TextButton(
                onClick = onContinue,
                modifier = Modifier.align(Alignment.CenterHorizontally),
            ) {
                Text(
                    text = stringResource(Res.string.auth_cleartext_continue),
                    color = MaterialTheme.colorScheme.error,
                )
            }

            Spacer(modifier = Modifier.height(IndelibleSpacing.step16))
        }
    }
}
