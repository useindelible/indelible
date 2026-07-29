package app.indelible.profile.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.Check
import androidx.compose.material.icons.filled.Close
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.unit.dp
import app.indelible.api.generated.models.MilaPromptPresetResponse
import app.indelible.api.generated.models.TestMilaConfigResponse
import app.indelible.profile.ui.components.PreferenceDropdownRow
import app.indelible.profile.ui.components.SettingsRow
import app.indelible.profile.ui.components.SettingsSection
import app.indelible.profile.ui.components.ToggleRow
import app.indelible.profile.viewmodel.AiSettingsUiState
import app.indelible.profile.viewmodel.AiSettingsViewModel
import app.indelible.ui.components.IndelibleButton
import app.indelible.ui.components.IndelibleButtonStyle
import app.indelible.ui.components.IndelibleTextField
import app.indelible.ui.theme.IndelibleSpacing

private val COMMON_CHAT_MODELS =
    listOf(
        "qwen-long",
        "qwen-plus",
        "qwen-max",
        "qwen-turbo",
        "gpt-4o",
        "gpt-4o-mini",
        "gpt-4-turbo",
        "gpt-3.5-turbo",
        "mistral-medium-latest",
        "llama-3.1-70b-instruct",
    )

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun AiSettingsScreen(
    viewModel: AiSettingsViewModel,
    onNavigateBack: () -> Unit,
    onNavigateToPreset: (presetId: String?) -> Unit,
    modifier: Modifier = Modifier,
) {
    val state by viewModel.uiState.collectAsState()

    Scaffold(
        modifier = modifier,
        topBar = {
            TopAppBar(
                title = {
                    Text(
                        text = "Mila & AI",
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
    ) { paddingValues ->
        if (state.isLoading) {
            Column(
                modifier =
                    Modifier
                        .fillMaxSize()
                        .padding(top = paddingValues.calculateTopPadding()),
                horizontalAlignment = Alignment.CenterHorizontally,
                verticalArrangement = Arrangement.Center,
            ) {
                CircularProgressIndicator()
            }
            return@Scaffold
        }

        Column(
            modifier =
                Modifier
                    .fillMaxSize()
                    .padding(top = paddingValues.calculateTopPadding())
                    .verticalScroll(rememberScrollState()),
        ) {
            // ── Mila ──────────────────────────────────────────────────────
            SettingsSection(title = "Mila") {
                ToggleRow(
                    label = "Enable Mila",
                    sublabel = "AI summaries, auto-tags, and reading assistant",
                    checked = state.enabled,
                    onCheckedChange = { viewModel.toggleEnabled(it) },
                )
            }

            // ── Provider ──────────────────────────────────────────────────
            ProviderConfigSection(state = state, viewModel = viewModel)

            // ── Models ────────────────────────────────────────────────────
            SettingsSection(title = "Models") {
                val modelOptions =
                    buildList {
                        addAll(COMMON_CHAT_MODELS)
                        if (state.chatModel.isNotBlank() && state.chatModel !in COMMON_CHAT_MODELS) {
                            add(0, state.chatModel)
                        }
                    }
                PreferenceDropdownRow(
                    label = "Chat Model",
                    currentValue = state.chatModel,
                    displayName = { it },
                    options = modelOptions,
                    onSelected = { viewModel.updateChatModel(it) },
                )
            }

            // ── Prompt Presets ─────────────────────────────────────────────
            PromptPresetListSection(
                state = state,
                onNavigateToPreset = onNavigateToPreset,
                viewModel = viewModel,
            )

            // ── Save ──────────────────────────────────────────────────────
            Column(
                modifier =
                    Modifier.padding(
                        horizontal = IndelibleSpacing.screenPaddingH,
                        vertical = IndelibleSpacing.step24,
                    ),
                verticalArrangement = Arrangement.spacedBy(IndelibleSpacing.step8),
            ) {
                if (state.saveError != null) {
                    Text(
                        text = state.saveError!!,
                        color = MaterialTheme.colorScheme.error,
                        style = MaterialTheme.typography.bodySmall,
                    )
                }
                IndelibleButton(
                    text = if (state.reindexConfirmationRequired) "Rebuild embeddings" else "Save",
                    onClick = { viewModel.save() },
                    isLoading = state.isSaving,
                )
            }

            Spacer(modifier = Modifier.height(IndelibleSpacing.step32))
        }
    }
}

@Composable
private fun ProviderConfigSection(
    state: AiSettingsUiState,
    viewModel: AiSettingsViewModel,
) {
    SettingsSection(title = "Provider") {
        Column(
            modifier =
                Modifier
                    .fillMaxWidth()
                    .padding(
                        horizontal = IndelibleSpacing.screenPaddingH,
                        vertical = IndelibleSpacing.step4,
                    ),
            verticalArrangement = Arrangement.spacedBy(IndelibleSpacing.step12),
        ) {
            IndelibleTextField(
                value = state.apiBase,
                onValueChange = { viewModel.updateApiBase(it) },
                label = "API Base URL",
                imeAction = ImeAction.Next,
            )
            IndelibleTextField(
                value = state.apiKey,
                onValueChange = { viewModel.updateApiKey(it) },
                label = if (state.hasApiKey) "API Key (leave blank to keep)" else "API Key",
                isPassword = true,
                imeAction = ImeAction.Done,
            )
            if (state.testResult != null) {
                TestResultBanner(result = state.testResult)
            }
            IndelibleButton(
                text = "Test Connection",
                onClick = { viewModel.testConnection() },
                isLoading = state.isTesting,
                style = IndelibleButtonStyle.Text,
                modifier = Modifier.fillMaxWidth(),
            )
        }
    }
}

@Composable
private fun PromptPresetListSection(
    state: AiSettingsUiState,
    onNavigateToPreset: (presetId: String?) -> Unit,
    viewModel: AiSettingsViewModel,
) {
    SettingsSection(title = "Prompt Presets") {
        Column(
            modifier =
                Modifier
                    .fillMaxWidth()
                    .background(MaterialTheme.colorScheme.surface),
        ) {
            state.presets.forEachIndexed { index, preset ->
                PromptPresetRow(
                    preset = preset,
                    onClick = { onNavigateToPreset(preset.id) },
                    onDeleteClick = { preset.id?.let { viewModel.deletePreset(it) } },
                )
                if (index < state.presets.lastIndex) {
                    HorizontalDivider(
                        color = MaterialTheme.colorScheme.outlineVariant,
                        modifier = Modifier.padding(start = IndelibleSpacing.rowPaddingH),
                    )
                }
            }
            if (state.presets.isNotEmpty()) {
                HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant)
            }
            SettingsRow(
                label = "+ Add Preset",
                onClick = { onNavigateToPreset(null) },
                showChevron = false,
                labelColor = MaterialTheme.colorScheme.primary,
            )
        }
    }
}

@Composable
private fun TestResultBanner(result: TestMilaConfigResponse) {
    val isSuccess = result.success
    val bgColor =
        if (isSuccess) {
            MaterialTheme.colorScheme.primaryContainer
        } else {
            MaterialTheme.colorScheme.errorContainer
        }
    val textColor =
        if (isSuccess) {
            MaterialTheme.colorScheme.onPrimaryContainer
        } else {
            MaterialTheme.colorScheme.onErrorContainer
        }
    Row(
        modifier =
            Modifier
                .fillMaxWidth()
                .background(bgColor, MaterialTheme.shapes.small)
                .padding(
                    horizontal = IndelibleSpacing.step12,
                    vertical = IndelibleSpacing.step10,
                ),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step8),
    ) {
        Icon(
            imageVector = if (isSuccess) Icons.Filled.Check else Icons.Filled.Close,
            contentDescription = null,
            tint = textColor,
            modifier = Modifier.size(IndelibleSpacing.step16),
        )
        Column {
            Text(
                text = if (isSuccess) "Connection successful" else "Connection failed",
                style = MaterialTheme.typography.bodySmall,
                color = textColor,
            )
            if (!result.error.isNullOrBlank()) {
                Text(
                    text = result.error,
                    style = MaterialTheme.typography.bodySmall,
                    color = textColor,
                )
            }
        }
    }
}

@OptIn(ExperimentalLayoutApi::class)
@Composable
private fun PromptPresetRow(
    preset: MilaPromptPresetResponse,
    onClick: () -> Unit,
    onDeleteClick: () -> Unit,
) {
    Row(
        modifier =
            Modifier
                .fillMaxWidth()
                .clickable(onClick = onClick)
                .padding(horizontal = IndelibleSpacing.rowPaddingH, vertical = IndelibleSpacing.rowPaddingV),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Column(modifier = Modifier.weight(1f)) {
            Text(
                text = preset.name,
                style = MaterialTheme.typography.bodyMedium,
            )
            FlowRow(
                modifier = Modifier.padding(top = IndelibleSpacing.step4),
                horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step4),
                verticalArrangement = Arrangement.spacedBy(IndelibleSpacing.step4),
            ) {
                PresetChip(label = preset.action)
                if (preset.isDefault) PresetChip(label = "default", isPrimary = true)
                if (preset.isBuiltIn) PresetChip(label = "built-in")
            }
        }
        if (!preset.isBuiltIn) {
            IconButton(
                onClick = onDeleteClick,
                modifier = Modifier.size(IndelibleSpacing.step32),
            ) {
                Icon(
                    imageVector = Icons.Filled.Delete,
                    contentDescription = "Delete preset",
                    tint = MaterialTheme.colorScheme.error,
                    modifier = Modifier.size(IndelibleSpacing.step20),
                )
            }
        }
    }
}

@Composable
private fun PresetChip(
    label: String,
    isPrimary: Boolean = false,
) {
    val bgColor =
        if (isPrimary) {
            MaterialTheme.colorScheme.primaryContainer
        } else {
            MaterialTheme.colorScheme.surfaceVariant
        }
    val textColor =
        if (isPrimary) {
            MaterialTheme.colorScheme.onPrimaryContainer
        } else {
            MaterialTheme.colorScheme.onSurfaceVariant
        }
    Text(
        text = label,
        style = MaterialTheme.typography.labelSmall,
        color = textColor,
        modifier =
            Modifier
                .background(bgColor, RoundedCornerShape(4.dp))
                .padding(horizontal = IndelibleSpacing.step6, vertical = IndelibleSpacing.step2),
    )
}
