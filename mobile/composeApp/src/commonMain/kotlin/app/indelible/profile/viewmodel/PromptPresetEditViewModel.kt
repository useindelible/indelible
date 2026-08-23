package app.indelible.profile.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import app.indelible.api.generated.models.CreateMilaPromptPresetBody
import app.indelible.api.generated.models.MilaPromptPresetResponse
import app.indelible.api.generated.models.UpdateMilaPromptPresetBody
import app.indelible.core.i18n.UiMessage
import app.indelible.profile.repository.MilaSettingsRepository
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.mila_name_required
import indelible.composeapp.generated.resources.mila_preset_delete_failed
import indelible.composeapp.generated.resources.mila_preset_save_failed
import indelible.composeapp.generated.resources.mila_system_prompt_required
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

data class PromptPresetEditUiState(
    val isLoading: Boolean = false,
    val isSaving: Boolean = false,
    val isDeleting: Boolean = false,
    val name: String = "",
    val action: String = "summary",
    val systemPrompt: String = "",
    val isDefault: Boolean = false,
    val isBuiltIn: Boolean = false,
    val saveError: UiMessage? = null,
    val isDone: Boolean = false,
)

val PRESET_ACTIONS = listOf("summary", "tags", "entities", "chat")

class PromptPresetEditViewModel(
    private val repository: MilaSettingsRepository,
    private val existingPreset: MilaPromptPresetResponse?,
) : ViewModel() {
    val isExistingPreset: Boolean get() = existingPreset?.id != null
    private val _uiState =
        MutableStateFlow(
            if (existingPreset != null) {
                PromptPresetEditUiState(
                    name = existingPreset.name,
                    action = existingPreset.action,
                    systemPrompt = existingPreset.systemPrompt,
                    isDefault = existingPreset.isDefault,
                    isBuiltIn = existingPreset.isBuiltIn,
                )
            } else {
                PromptPresetEditUiState()
            },
        )
    val uiState: StateFlow<PromptPresetEditUiState> = _uiState.asStateFlow()

    fun updateName(value: String) {
        _uiState.value = _uiState.value.copy(name = value, saveError = null)
    }

    fun updateAction(value: String) {
        _uiState.value = _uiState.value.copy(action = value, saveError = null)
    }

    fun updateSystemPrompt(value: String) {
        _uiState.value = _uiState.value.copy(systemPrompt = value, saveError = null)
    }

    fun updateIsDefault(value: Boolean) {
        _uiState.value = _uiState.value.copy(isDefault = value)
    }

    fun save() {
        val state = _uiState.value
        if (state.name.isBlank()) {
            _uiState.value = state.copy(saveError = UiMessage(Res.string.mila_name_required))
            return
        }
        if (state.systemPrompt.isBlank()) {
            _uiState.value = state.copy(saveError = UiMessage(Res.string.mila_system_prompt_required))
            return
        }
        viewModelScope.launch {
            _uiState.value = _uiState.value.copy(isSaving = true, saveError = null)
            val result =
                if (existingPreset?.id != null) {
                    repository.updatePromptPreset(
                        existingPreset.id,
                        UpdateMilaPromptPresetBody(
                            name = state.name,
                            systemPrompt = state.systemPrompt,
                            isDefault = state.isDefault,
                        ),
                    )
                } else {
                    repository.createPromptPreset(
                        CreateMilaPromptPresetBody(
                            action = state.action,
                            name = state.name,
                            systemPrompt = state.systemPrompt,
                            isDefault = state.isDefault,
                        ),
                    )
                }
            result
                .onSuccess {
                    _uiState.value = _uiState.value.copy(isSaving = false, isDone = true)
                }.onFailure {
                    _uiState.value =
                        _uiState.value.copy(
                            isSaving = false,
                            saveError = UiMessage(Res.string.mila_preset_save_failed),
                        )
                }
        }
    }

    fun delete() {
        val presetId = existingPreset?.id ?: return
        viewModelScope.launch {
            _uiState.value = _uiState.value.copy(isDeleting = true)
            repository
                .deletePromptPreset(presetId)
                .onSuccess {
                    _uiState.value = _uiState.value.copy(isDeleting = false, isDone = true)
                }.onFailure {
                    _uiState.value =
                        _uiState.value.copy(
                            isDeleting = false,
                            saveError = UiMessage(Res.string.mila_preset_delete_failed),
                        )
                }
        }
    }
}
