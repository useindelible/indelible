package app.indelible.profile.ui

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
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.input.ImeAction
import app.indelible.profile.ui.components.PreferenceDropdownRow
import app.indelible.profile.ui.components.SettingsSection
import app.indelible.profile.ui.components.ToggleRow
import app.indelible.profile.viewmodel.PRESET_ACTIONS
import app.indelible.profile.viewmodel.PromptPresetEditViewModel
import app.indelible.ui.components.IndelibleButton
import app.indelible.ui.components.IndelibleButtonStyle
import app.indelible.ui.components.IndelibleTextField
import app.indelible.ui.theme.IndelibleSpacing

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun PromptPresetEditScreen(
    viewModel: PromptPresetEditViewModel,
    onNavigateBack: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val state by viewModel.uiState.collectAsState()

    LaunchedEffect(state.isDone) {
        if (state.isDone) onNavigateBack()
    }

    val isEditing = !state.isBuiltIn

    Scaffold(
        modifier = modifier,
        topBar = {
            TopAppBar(
                title = {
                    Text(
                        text =
                            if (state.isBuiltIn) {
                                state.name
                            } else if (state.name.isBlank()) {
                                "New Preset"
                            } else {
                                state.name
                            },
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
        Column(
            modifier =
                Modifier
                    .fillMaxSize()
                    .padding(top = paddingValues.calculateTopPadding())
                    .verticalScroll(rememberScrollState()),
        ) {
            SettingsSection(title = "Preset") {
                IndelibleTextField(
                    value = state.name,
                    onValueChange = { viewModel.updateName(it) },
                    label = "Name",
                    enabled = isEditing,
                    imeAction = ImeAction.Next,
                    modifier =
                        Modifier.padding(
                            horizontal = IndelibleSpacing.rowPaddingH,
                            vertical = IndelibleSpacing.step12,
                        ),
                )
                HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant)
                PreferenceDropdownRow(
                    label = "Action",
                    currentValue = state.action,
                    displayName = { it.replaceFirstChar { c -> c.uppercase() } },
                    options = PRESET_ACTIONS,
                    onSelected = { if (!viewModel.isExistingPreset) viewModel.updateAction(it) },
                )
                HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant)
                ToggleRow(
                    label = "Set as Default",
                    sublabel = "Use this preset by default for the ${state.action} action",
                    checked = state.isDefault,
                    onCheckedChange = { if (isEditing) viewModel.updateIsDefault(it) },
                )
            }

            SettingsSection(title = "System Prompt") {
                IndelibleTextField(
                    value = state.systemPrompt,
                    onValueChange = { viewModel.updateSystemPrompt(it) },
                    label = "System Prompt",
                    enabled = isEditing,
                    singleLine = false,
                    minLines = 6,
                    maxLines = 12,
                    imeAction = ImeAction.Default,
                    modifier =
                        Modifier.padding(
                            horizontal = IndelibleSpacing.rowPaddingH,
                            vertical = IndelibleSpacing.step12,
                        ),
                )
            }

            if (state.saveError != null) {
                Text(
                    text = state.saveError!!,
                    color = MaterialTheme.colorScheme.error,
                    style = MaterialTheme.typography.bodySmall,
                    modifier =
                        Modifier.padding(
                            horizontal = IndelibleSpacing.screenPaddingH,
                            vertical = IndelibleSpacing.step8,
                        ),
                )
            }

            if (isEditing) {
                Column(
                    modifier =
                        Modifier.padding(
                            horizontal = IndelibleSpacing.screenPaddingH,
                            vertical = IndelibleSpacing.step24,
                        ),
                    verticalArrangement = Arrangement.spacedBy(IndelibleSpacing.step12),
                ) {
                    IndelibleButton(
                        text = "Save",
                        onClick = { viewModel.save() },
                        isLoading = state.isSaving,
                        modifier = Modifier.fillMaxWidth(),
                    )
                    if (state.action.isNotBlank() && !state.isBuiltIn && viewModel.isExistingPreset) {
                        IndelibleButton(
                            text = "Delete Preset",
                            onClick = { viewModel.delete() },
                            isLoading = state.isDeleting,
                            style = IndelibleButtonStyle.Destructive,
                            modifier = Modifier.fillMaxWidth(),
                        )
                    }
                }
            }

            Spacer(modifier = Modifier.height(IndelibleSpacing.step32))
        }
    }
}
