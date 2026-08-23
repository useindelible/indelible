package app.indelible.profile.ui

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
import androidx.compose.material3.Button
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import app.indelible.auth.viewmodel.AuthState
import app.indelible.auth.viewmodel.AuthViewModel
import app.indelible.profile.ui.components.UserAvatar
import app.indelible.ui.theme.IndelibleSpacing
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.common_back
import indelible.composeapp.generated.resources.common_save
import indelible.composeapp.generated.resources.profile_display_name
import indelible.composeapp.generated.resources.profile_edit_title
import indelible.composeapp.generated.resources.profile_email
import indelible.composeapp.generated.resources.profile_saving
import indelible.composeapp.generated.resources.profile_update_failed
import kotlinx.coroutines.launch
import org.jetbrains.compose.resources.stringResource

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ProfileEditScreen(
    authViewModel: AuthViewModel,
    onNavigateBack: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val authState by authViewModel.authState.collectAsState()
    val user = (authState as? AuthState.Authenticated)?.user
    val avatarBytes by authViewModel.avatarBytes.collectAsState()
    var displayName by remember { mutableStateOf(user?.displayName ?: "") }
    var isSaving by remember { mutableStateOf(false) }
    val snackbarHostState = remember { SnackbarHostState() }
    val coroutineScope = rememberCoroutineScope()
    val updateFailedMessage = stringResource(Res.string.profile_update_failed)

    Scaffold(
        modifier = modifier,
        topBar = {
            TopAppBar(
                title = { Text(stringResource(Res.string.profile_edit_title)) },
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
                    .padding(paddingValues)
                    .verticalScroll(rememberScrollState())
                    .padding(IndelibleSpacing.step24),
            horizontalAlignment = Alignment.CenterHorizontally,
        ) {
            UserAvatar(
                displayName = displayName,
                avatarUrl = user?.avatarUrl,
                avatarBytes = avatarBytes,
                size = IndelibleSpacing.step96,
                textStyle = MaterialTheme.typography.headlineLarge,
            )

            Spacer(modifier = Modifier.height(IndelibleSpacing.step24))

            OutlinedTextField(
                value = displayName,
                onValueChange = { displayName = it },
                label = { Text(stringResource(Res.string.profile_display_name)) },
                singleLine = true,
                modifier = Modifier.fillMaxWidth(),
            )

            Spacer(modifier = Modifier.height(IndelibleSpacing.step12))

            OutlinedTextField(
                value = user?.email ?: "",
                onValueChange = {},
                label = { Text(stringResource(Res.string.profile_email)) },
                singleLine = true,
                enabled = false,
                modifier = Modifier.fillMaxWidth(),
            )

            Spacer(modifier = Modifier.height(IndelibleSpacing.step32))

            Button(
                onClick = {
                    isSaving = true
                    authViewModel.updateProfile(displayName) { succeeded ->
                        isSaving = false
                        if (succeeded) {
                            onNavigateBack()
                        } else {
                            coroutineScope.launch {
                                snackbarHostState.showSnackbar(updateFailedMessage)
                            }
                        }
                    }
                },
                enabled = !isSaving,
                modifier = Modifier.fillMaxWidth(),
            ) {
                Text(
                    stringResource(
                        if (isSaving) Res.string.profile_saving else Res.string.common_save,
                    ),
                )
            }
        }
    }
}
