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
import app.indelible.core.i18n.resolve
import app.indelible.profile.ui.components.PreferenceDropdownRow
import app.indelible.profile.ui.components.SettingsSection
import app.indelible.profile.ui.components.ToggleRow
import app.indelible.profile.viewmodel.PRESET_ACTIONS
import app.indelible.profile.viewmodel.PromptPresetEditViewModel
import app.indelible.ui.components.IndelibleButton
import app.indelible.ui.components.IndelibleButtonStyle
import app.indelible.ui.components.IndelibleTextField
import app.indelible.ui.theme.IndelibleSpacing
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.common_back
import indelible.composeapp.generated.resources.common_save
import indelible.composeapp.generated.resources.mila_delete_preset
import indelible.composeapp.generated.resources.mila_name
import indelible.composeapp.generated.resources.mila_new_preset
import indelible.composeapp.generated.resources.mila_preset
import indelible.composeapp.generated.resources.mila_preset_action
import indelible.composeapp.generated.resources.mila_preset_default_description
import indelible.composeapp.generated.resources.mila_set_default
import indelible.composeapp.generated.resources.mila_system_prompt
import org.jetbrains.compose.resources.stringResource

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
                                stringResource(Res.string.mila_new_preset)
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
                            contentDescription = stringResource(Res.string.common_back),
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
            SettingsSection(title = stringResource(Res.string.mila_preset)) {
                IndelibleTextField(
                    value = state.name,
                    onValueChange = { viewModel.updateName(it) },
                    label = stringResource(Res.string.mila_name),
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
                    label = stringResource(Res.string.mila_preset_action),
                    currentValue = state.action,
                    displayName = { presetActionLabel(it) },
                    options = PRESET_ACTIONS,
                    onSelected = { if (!viewModel.isExistingPreset) viewModel.updateAction(it) },
                )
                HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant)
                ToggleRow(
                    label = stringResource(Res.string.mila_set_default),
                    sublabel =
                        stringResource(
                            Res.string.mila_preset_default_description,
                            presetActionLabel(state.action),
                        ),
                    checked = state.isDefault,
                    onCheckedChange = { if (isEditing) viewModel.updateIsDefault(it) },
                )
            }

            SettingsSection(title = stringResource(Res.string.mila_system_prompt)) {
                IndelibleTextField(
                    value = state.systemPrompt,
                    onValueChange = { viewModel.updateSystemPrompt(it) },
                    label = stringResource(Res.string.mila_system_prompt),
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
                    text = state.saveError!!.resolve(),
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
                        text = stringResource(Res.string.common_save),
                        onClick = { viewModel.save() },
                        isLoading = state.isSaving,
                        modifier = Modifier.fillMaxWidth(),
                    )
                    if (state.action.isNotBlank() && !state.isBuiltIn && viewModel.isExistingPreset) {
                        IndelibleButton(
                            text = stringResource(Res.string.mila_delete_preset),
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
