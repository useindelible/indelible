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
import app.indelible.profile.ui.components.SettingsRow
import app.indelible.profile.ui.components.SettingsSection
import app.indelible.profile.viewmodel.AccountEffect
import app.indelible.profile.viewmodel.AccountViewModel
import app.indelible.ui.components.IndelibleButton
import app.indelible.ui.components.IndelibleButtonStyle
import app.indelible.ui.components.IndelibleTextField
import app.indelible.ui.theme.IndelibleSpacing

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
                is AccountEffect.ShowSnackbar -> snackbarHostState.showSnackbar(effect.message)
                is AccountEffect.AccountDeleted -> onAccountDeleted()
            }
        }
    }

    if (showSignOutDialog) {
        AlertDialog(
            onDismissRequest = { showSignOutDialog = false },
            title = { Text("Sign Out") },
            text = { Text("Are you sure you want to sign out?") },
            confirmButton = {
                TextButton(
                    onClick = {
                        showSignOutDialog = false
                        onSignOut()
                    },
                ) { Text("Sign Out") }
            },
            dismissButton = {
                TextButton(onClick = { showSignOutDialog = false }) { Text("Cancel") }
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
                        text = "Account",
                        style = MaterialTheme.typography.headlineSmall,
                    )
                },
                navigationIcon = {
                    IconButton(onClick = onNavigateBack) {
                        Icon(
                            imageVector = Icons.AutoMirrored.Filled.ArrowBack,
                            contentDescription = "Back",
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
    SettingsSection(title = "Account") {
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
                label = "Storage Usage",
                sublabel = "Library + archives",
                onClick = {},
                value = "—",
                showChevron = false,
            )
            HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant)
            SettingsRow(
                label = "Change Password",
                onClick = onNavigateToChangePassword,
            )
            HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant)
            SettingsRow(
                label = "Sign Out",
                onClick = onSignOutClick,
                showChevron = false,
                labelColor = MaterialTheme.colorScheme.error,
            )
        }
    }
}

@Composable
private fun DangerZoneCard(
    onDeleteAccountClick: () -> Unit,
) {
    SettingsSection(title = "Danger Zone") {
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
                    text = "Delete Account",
                    style = MaterialTheme.typography.titleSmall.copy(fontWeight = FontWeight.SemiBold),
                    color = MaterialTheme.colorScheme.error,
                )
                Text(
                    text = "Permanently removes your account, library, and all archives. " +
                        "This action cannot be undone.",
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
                Row(horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step8)) {
                    IndelibleButton(
                        text = "Export Data First",
                        onClick = {},
                        style = IndelibleButtonStyle.Secondary,
                        compact = true,
                    )
                    IndelibleButton(
                        text = "Delete Account",
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
        title = { Text("Delete Account") },
        text = {
            Column(verticalArrangement = Arrangement.spacedBy(IndelibleSpacing.step8)) {
                Text(
                    text = "This action is permanent and cannot be undone. Type DELETE to confirm.",
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
                IndelibleTextField(
                    value = input,
                    onValueChange = { input = it },
                    label = "Type DELETE to confirm",
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
                    text = "Delete Account",
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
            TextButton(onClick = onDismiss) { Text("Cancel") }
        },
    )
}
