package app.indelible.profile.ui

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import app.indelible.core.i18n.resolveString
import app.indelible.profile.ui.components.SettingsRow
import app.indelible.profile.ui.components.SettingsSection
import app.indelible.profile.viewmodel.AccountEffect
import app.indelible.profile.viewmodel.AccountViewModel
import app.indelible.ui.components.IndelibleButton
import app.indelible.ui.components.IndelibleButtonStyle
import app.indelible.ui.components.IndelibleTextField
import app.indelible.ui.theme.IndelibleSpacing
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.common_back
import indelible.composeapp.generated.resources.common_cancel
import indelible.composeapp.generated.resources.profile_account
import indelible.composeapp.generated.resources.profile_change_password
import indelible.composeapp.generated.resources.profile_danger_zone
import indelible.composeapp.generated.resources.profile_delete_account
import indelible.composeapp.generated.resources.profile_delete_confirm_instruction
import indelible.composeapp.generated.resources.profile_delete_confirm_label
import indelible.composeapp.generated.resources.profile_delete_description
import indelible.composeapp.generated.resources.profile_export_data_first
import indelible.composeapp.generated.resources.profile_sign_out
import indelible.composeapp.generated.resources.profile_sign_out_confirm
import indelible.composeapp.generated.resources.profile_storage_usage
import indelible.composeapp.generated.resources.profile_storage_usage_description
import org.jetbrains.compose.resources.stringResource

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun AccountScreen(
    viewModel: AccountViewModel,
    onNavigateBack: () -> Unit,
    onNavigateToChangePassword: () -> Unit,
    onSignOut: () -> Unit,
    onAccountDeleted: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val snackbarHostState = remember { SnackbarHostState() }

    var showSignOutDialog by remember { mutableStateOf(false) }
    var showDeleteAccountDialog by remember { mutableStateOf(false) }

    LaunchedEffect(viewModel) {
        viewModel.effects.collect { effect ->
            when (effect) {
                is AccountEffect.ShowSnackbar -> snackbarHostState.showSnackbar(effect.message.resolveString())
                is AccountEffect.AccountDeleted -> onAccountDeleted()
            }
        }
    }

    if (showSignOutDialog) {
        AlertDialog(
            onDismissRequest = { showSignOutDialog = false },
            title = { Text(stringResource(Res.string.profile_sign_out)) },
            text = { Text(stringResource(Res.string.profile_sign_out_confirm)) },
            confirmButton = {
                TextButton(
                    onClick = {
                        showSignOutDialog = false
                        onSignOut()
                    },
                ) { Text(stringResource(Res.string.profile_sign_out)) }
            },
            dismissButton = {
                TextButton(onClick = { showSignOutDialog = false }) {
                    Text(stringResource(Res.string.common_cancel))
                }
            },
        )
    }

    if (showDeleteAccountDialog) {
        DeleteAccountDialog(
            onConfirm = { confirmation ->
                showDeleteAccountDialog = false
                viewModel.deleteAccount(confirmation)
            },
            onDismiss = { showDeleteAccountDialog = false },
        )
    }

    Scaffold(
        modifier = modifier,
        topBar = {
            TopAppBar(
                title = {
                    Text(
                        text = stringResource(Res.string.profile_account),
                        style = MaterialTheme.typography.headlineSmall,
                    )
                },
                navigationIcon = {
                    IconButton(onClick = onNavigateBack) {
                        Icon(
                            imageVector = Icons.AutoMirrored.Filled.ArrowBack,
                            contentDescription = stringResource(Res.string.common_back),
                        )
                    }
                },
            )
        },
        snackbarHost = { SnackbarHost(snackbarHostState) },
    ) { paddingValues ->
        Column(
            modifier =
                Modifier
                    .fillMaxSize()
                    .padding(top = paddingValues.calculateTopPadding())
                    .verticalScroll(rememberScrollState()),
        ) {
            AccountSettingsCard(
                onNavigateToChangePassword = onNavigateToChangePassword,
                onSignOutClick = { showSignOutDialog = true },
            )
            DangerZoneCard(
                onDeleteAccountClick = { showDeleteAccountDialog = true },
            )
            Spacer(modifier = Modifier.height(IndelibleSpacing.step32))
        }
    }
}

@Composable
private fun AccountSettingsCard(
    onNavigateToChangePassword: () -> Unit,
    onSignOutClick: () -> Unit,
) {
    SettingsSection(title = stringResource(Res.string.profile_account)) {
        Card(
            modifier =
                Modifier
                    .fillMaxWidth()
                    .padding(horizontal = IndelibleSpacing.step16, vertical = IndelibleSpacing.step8),
            shape = MaterialTheme.shapes.extraLarge,
            colors =
                CardDefaults.cardColors(
                    containerColor = MaterialTheme.colorScheme.surfaceVariant,
                ),
            border = BorderStroke(0.5.dp, MaterialTheme.colorScheme.outlineVariant),
            elevation = CardDefaults.cardElevation(defaultElevation = 0.dp),
        ) {
            SettingsRow(
                label = stringResource(Res.string.profile_storage_usage),
                sublabel = stringResource(Res.string.profile_storage_usage_description),
                onClick = {},
                value = "—",
                showChevron = false,
            )
            HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant)
            SettingsRow(
                label = stringResource(Res.string.profile_change_password),
                onClick = onNavigateToChangePassword,
            )
            HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant)
            SettingsRow(
                label = stringResource(Res.string.profile_sign_out),
                onClick = onSignOutClick,
                showChevron = false,
                labelColor = MaterialTheme.colorScheme.error,
            )
        }
    }
}

@Composable
private fun DangerZoneCard(onDeleteAccountClick: () -> Unit) {
    SettingsSection(title = stringResource(Res.string.profile_danger_zone)) {
        Card(
            modifier =
                Modifier
                    .fillMaxWidth()
                    .padding(horizontal = IndelibleSpacing.step16, vertical = IndelibleSpacing.step8),
            shape = MaterialTheme.shapes.extraLarge,
            colors =
                CardDefaults.cardColors(
                    containerColor = MaterialTheme.colorScheme.surfaceVariant,
                ),
            border = BorderStroke(0.5.dp, MaterialTheme.colorScheme.outlineVariant),
            elevation = CardDefaults.cardElevation(defaultElevation = 0.dp),
        ) {
            Column(
                modifier =
                    Modifier
                        .fillMaxWidth()
                        .padding(IndelibleSpacing.step16),
                verticalArrangement = Arrangement.spacedBy(IndelibleSpacing.step10),
            ) {
                Text(
                    text = stringResource(Res.string.profile_delete_account),
                    style = MaterialTheme.typography.titleSmall.copy(fontWeight = FontWeight.SemiBold),
                    color = MaterialTheme.colorScheme.error,
                )
                Text(
                    text = stringResource(Res.string.profile_delete_description),
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
                Row(horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step8)) {
                    IndelibleButton(
                        text = stringResource(Res.string.profile_export_data_first),
                        onClick = {},
                        style = IndelibleButtonStyle.Secondary,
                        compact = true,
                    )
                    IndelibleButton(
                        text = stringResource(Res.string.profile_delete_account),
                        onClick = onDeleteAccountClick,
                        style = IndelibleButtonStyle.OutlinedDestructive,
                        compact = true,
                    )
                }
            }
        }
    }
}

@Composable
private fun DeleteAccountDialog(
    onConfirm: (confirmation: String) -> Unit,
    onDismiss: () -> Unit,
) {
    var input by remember { mutableStateOf("") }
    val required = "DELETE"

    AlertDialog(
        onDismissRequest = onDismiss,
        title = { Text(stringResource(Res.string.profile_delete_account)) },
        text = {
            Column(verticalArrangement = Arrangement.spacedBy(IndelibleSpacing.step8)) {
                Text(
                    text = stringResource(Res.string.profile_delete_confirm_instruction),
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
                IndelibleTextField(
                    value = input,
                    onValueChange = { input = it },
                    label = stringResource(Res.string.profile_delete_confirm_label),
                    modifier = Modifier.fillMaxWidth(),
                )
            }
        },
        confirmButton = {
            TextButton(
                onClick = { onConfirm(input) },
                enabled = input == required,
            ) {
                Text(
                    text = stringResource(Res.string.profile_delete_account),
                    color =
                        if (input == required) {
                            MaterialTheme.colorScheme.error
                        } else {
                            MaterialTheme.colorScheme.onSurface.copy(alpha = 0.38f)
                        },
                )
            }
        },
        dismissButton = {
            TextButton(onClick = onDismiss) { Text(stringResource(Res.string.common_cancel)) }
        },
    )
}
