package app.indelible.profile.ui

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.derivedStateOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.unit.dp
import app.indelible.core.i18n.resolveString
import app.indelible.profile.viewmodel.ChangePasswordEffect
import app.indelible.profile.viewmodel.ChangePasswordViewModel
import app.indelible.ui.components.IndelibleButton
import app.indelible.ui.components.IndelibleTextField
import app.indelible.ui.theme.IndelibleSpacing
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.common_back
import indelible.composeapp.generated.resources.profile_change_password
import indelible.composeapp.generated.resources.profile_confirm_password
import indelible.composeapp.generated.resources.profile_current_password
import indelible.composeapp.generated.resources.profile_new_password
import indelible.composeapp.generated.resources.profile_password_min_length
import indelible.composeapp.generated.resources.profile_password_mismatch
import org.jetbrains.compose.resources.stringResource

private const val MIN_PASSWORD_LENGTH = 8

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ChangePasswordScreen(
    viewModel: ChangePasswordViewModel,
    onNavigateBack: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val isLoading by viewModel.isLoading.collectAsState()
    val snackbarHostState = remember { SnackbarHostState() }

    var currentPassword by remember { mutableStateOf("") }
    var newPassword by remember { mutableStateOf("") }
    var confirmPassword by remember { mutableStateOf("") }
    val minimumLengthError = stringResource(Res.string.profile_password_min_length, MIN_PASSWORD_LENGTH)
    val mismatchError = stringResource(Res.string.profile_password_mismatch)

    val newPasswordError by remember {
        derivedStateOf {
            if (newPassword.isNotEmpty() && newPassword.length < MIN_PASSWORD_LENGTH) {
                minimumLengthError
            } else {
                null
            }
        }
    }
    val confirmPasswordError by remember {
        derivedStateOf {
            if (confirmPassword.isNotEmpty() && confirmPassword != newPassword) {
                mismatchError
            } else {
                null
            }
        }
    }
    val canSubmit by remember {
        derivedStateOf {
            currentPassword.isNotBlank() &&
                newPassword.length >= MIN_PASSWORD_LENGTH &&
                confirmPassword == newPassword
        }
    }

    LaunchedEffect(viewModel) {
        viewModel.effects.collect { effect ->
            when (effect) {
                is ChangePasswordEffect.ShowSnackbar -> snackbarHostState.showSnackbar(effect.message.resolveString())
                is ChangePasswordEffect.NavigateBack -> onNavigateBack()
            }
        }
    }

    Scaffold(
        modifier = modifier,
        topBar = {
            TopAppBar(
                title = {
                    Text(
                        text = stringResource(Res.string.profile_change_password),
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
            Card(
                modifier =
                    Modifier
                        .fillMaxWidth()
                        .padding(horizontal = IndelibleSpacing.step16, vertical = IndelibleSpacing.step16),
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
                    verticalArrangement = Arrangement.spacedBy(IndelibleSpacing.step16),
                ) {
                    IndelibleTextField(
                        value = currentPassword,
                        onValueChange = { currentPassword = it },
                        label = stringResource(Res.string.profile_current_password),
                        isPassword = true,
                        keyboardType = KeyboardType.Password,
                        imeAction = ImeAction.Next,
                        modifier = Modifier.fillMaxWidth(),
                    )
                    IndelibleTextField(
                        value = newPassword,
                        onValueChange = { newPassword = it },
                        label = stringResource(Res.string.profile_new_password),
                        isPassword = true,
                        keyboardType = KeyboardType.Password,
                        imeAction = ImeAction.Next,
                        error = newPasswordError,
                        modifier = Modifier.fillMaxWidth(),
                    )
                    IndelibleTextField(
                        value = confirmPassword,
                        onValueChange = { confirmPassword = it },
                        label = stringResource(Res.string.profile_confirm_password),
                        isPassword = true,
                        keyboardType = KeyboardType.Password,
                        imeAction = ImeAction.Done,
                        onImeAction = { if (canSubmit) viewModel.changePassword(currentPassword, newPassword) },
                        error = confirmPasswordError,
                        modifier = Modifier.fillMaxWidth(),
                    )
                    IndelibleButton(
                        text = stringResource(Res.string.profile_change_password),
                        onClick = { viewModel.changePassword(currentPassword, newPassword) },
                        isLoading = isLoading,
                        enabled = canSubmit,
                        modifier = Modifier.fillMaxWidth(),
                    )
                }
            }

            Spacer(modifier = Modifier.height(IndelibleSpacing.step32))
        }
    }
}
