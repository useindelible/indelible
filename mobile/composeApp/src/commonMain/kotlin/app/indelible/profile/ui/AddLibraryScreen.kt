package app.indelible.profile.ui

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
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
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalClipboardManager
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.unit.dp
import app.indelible.profile.ui.components.SettingsSection
import app.indelible.profile.viewmodel.AddLibraryEffect
import app.indelible.profile.viewmodel.AddLibraryViewModel
import app.indelible.ui.components.IndelibleButton
import app.indelible.ui.components.IndelibleTextField
import app.indelible.ui.theme.IndelibleSpacing
import kotlinx.coroutines.launch

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun AddLibraryScreen(
    viewModel: AddLibraryViewModel,
    ingestLibraryEmail: String?,
    onNavigateBack: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val uiState by viewModel.uiState.collectAsState()
    var url by remember { mutableStateOf("") }
    val snackbarHostState = remember { SnackbarHostState() }
    val coroutineScope = rememberCoroutineScope()
    val clipboardManager = LocalClipboardManager.current

    LaunchedEffect(viewModel) {
        viewModel.reset()
        viewModel.effects.collect { effect ->
            when (effect) {
                AddLibraryEffect.Saved -> onNavigateBack()
            }
        }
    }

    Scaffold(
        modifier = modifier,
        topBar = {
            TopAppBar(
                title = {
                    Text(
                        text = "Add to Library",
                        style = MaterialTheme.typography.titleLarge,
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
                    .padding(paddingValues)
                    .padding(horizontal = IndelibleSpacing.screenPaddingH),
        ) {
            Spacer(modifier = Modifier.height(IndelibleSpacing.step16))

            SettingsSection(title = "Save to Library") {
                IndelibleTextField(
                    value = url,
                    onValueChange = {
                        url = it
                        viewModel.clearError()
                    },
                    label = "URL",
                    error = uiState.errorMessage,
                    enabled = !uiState.isSubmitting,
                    modifier = Modifier.fillMaxWidth(),
                )
                Spacer(modifier = Modifier.height(IndelibleSpacing.step16))
                IndelibleButton(
                    text = "Save to Library",
                    onClick = { viewModel.save(url) },
                    isLoading = uiState.isSubmitting,
                    enabled = url.isNotBlank(),
                )
            }

            if (ingestLibraryEmail != null) {
                Spacer(modifier = Modifier.height(IndelibleSpacing.sectionGap))

                SettingsSection(title = "Email Ingest") {
                    Card(
                        modifier = Modifier.fillMaxWidth(),
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
                                    .clickable {
                                        clipboardManager.setText(AnnotatedString(ingestLibraryEmail))
                                        coroutineScope.launch {
                                            snackbarHostState.showSnackbar("Email copied to clipboard")
                                        }
                                    }.padding(IndelibleSpacing.step16),
                        ) {
                            Text(
                                text = "Forward emails to save articles directly to your library.",
                                style = MaterialTheme.typography.bodyMedium,
                                color = MaterialTheme.colorScheme.onSurfaceVariant,
                            )
                            Spacer(modifier = Modifier.height(IndelibleSpacing.step8))
                            Text(
                                text = ingestLibraryEmail,
                                style = MaterialTheme.typography.titleSmall,
                                color = MaterialTheme.colorScheme.primary,
                            )
                            Spacer(modifier = Modifier.height(IndelibleSpacing.step4))
                            Text(
                                text = "Tap to copy",
                                style = MaterialTheme.typography.bodySmall,
                                color = MaterialTheme.colorScheme.onSurfaceVariant,
                            )
                        }
                    }
                }
            }
        }
    }
}
