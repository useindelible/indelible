package app.indelible.profile.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import app.indelible.api.generated.models.MilaPromptPresetResponse
import app.indelible.api.generated.models.TestMilaConfigBody
import app.indelible.api.generated.models.TestMilaConfigResponse
import app.indelible.api.generated.models.UpsertMilaConfigBody
import app.indelible.core.i18n.UiMessage
import app.indelible.profile.repository.MilaSettingsRepository
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.mila_connection_failed
import indelible.composeapp.generated.resources.mila_load_failed
import indelible.composeapp.generated.resources.mila_preset_delete_failed
import indelible.composeapp.generated.resources.mila_presets_load_failed
import indelible.composeapp.generated.resources.mila_rebuild_embeddings_confirm
import indelible.composeapp.generated.resources.mila_required_settings
import indelible.composeapp.generated.resources.mila_save_failed
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

private const val MILA_EMBEDDING_DIM = 768
private const val MILA_MODEL_CONTEXT_WINDOW = 16_000

data class AiSettingsUiState(
    val isLoading: Boolean = true,
    val enabled: Boolean = false,
    val apiBase: String = "",
    val embeddingApiBase: String = "",
    val apiKey: String = "",
    val hasApiKey: Boolean = false,
    val hasChatApiKey: Boolean = false,
    val hasEmbeddingApiKey: Boolean = false,
    val chatModel: String = "",
    val embeddingModel: String = "",
    val embeddingDim: Int = MILA_EMBEDDING_DIM,
    val modelContextWindow: Int = MILA_MODEL_CONTEXT_WINDOW,
    val savedEmbeddingModel: String = "",
    val savedEmbeddingDim: Int = MILA_EMBEDDING_DIM,
    val reindexConfirmationRequired: Boolean = false,
    val isSaving: Boolean = false,
    val isTesting: Boolean = false,
    val testResult: TestMilaConfigResponse? = null,
    val testError: UiMessage? = null,
    val saveError: UiMessage? = null,
    val presets: List<MilaPromptPresetResponse> = emptyList(),
)

class AiSettingsViewModel(
    private val repository: MilaSettingsRepository,
) : ViewModel() {
    private val _uiState = MutableStateFlow(AiSettingsUiState())
    val uiState: StateFlow<AiSettingsUiState> = _uiState.asStateFlow()

    fun refresh() {
        load()
    }

    private fun load() {
        viewModelScope.launch {
            _uiState.value = _uiState.value.copy(isLoading = true, saveError = null)
            repository
                .getConfig()
                .onSuccess { config ->
                    _uiState.value =
                        _uiState.value.copy(
                            isLoading = false,
                            enabled = config.enabled,
                            apiBase = config.chatApiBase,
                            embeddingApiBase = config.embeddingApiBase,
                            apiKey = "",
                            hasApiKey = config.hasChatApiKey || config.hasEmbeddingApiKey,
                            hasChatApiKey = config.hasChatApiKey,
                            hasEmbeddingApiKey = config.hasEmbeddingApiKey,
                            chatModel = config.chatModel,
                            embeddingModel = config.embeddingModel,
                            embeddingDim = config.embeddingDim,
                            modelContextWindow = config.modelContextWindow,
                            savedEmbeddingModel = config.embeddingModel,
                            savedEmbeddingDim = config.embeddingDim,
                            reindexConfirmationRequired = false,
                        )
                }.onFailure {
                    _uiState.value =
                        _uiState.value.copy(
                            isLoading = false,
                            saveError = UiMessage(Res.string.mila_load_failed),
                        )
                }
        }
        viewModelScope.launch {
            repository
                .getPromptPresets()
                .onSuccess { response ->
                    val flat = response.groups.flatMap { it.presets }
                    _uiState.value = _uiState.value.copy(presets = flat)
                }.onFailure {
                    _uiState.value =
                        _uiState.value.copy(saveError = UiMessage(Res.string.mila_presets_load_failed))
                }
        }
    }

    fun updateApiBase(value: String) {
        _uiState.value =
            _uiState.value.copy(
                apiBase = value,
                testResult = null,
                testError = null,
                saveError = null,
                reindexConfirmationRequired = false,
            )
    }

    fun updateApiKey(value: String) {
        _uiState.value =
            _uiState.value.copy(
                apiKey = value,
                testResult = null,
                testError = null,
                saveError = null,
                reindexConfirmationRequired = false,
            )
    }

    fun updateChatModel(value: String) {
        _uiState.value =
            _uiState.value.copy(
                chatModel = value,
                testResult = null,
                testError = null,
                saveError = null,
                reindexConfirmationRequired = false,
            )
    }

    fun toggleEnabled(enabled: Boolean) {
        val state = _uiState.value
        _uiState.value = state.copy(enabled = enabled)
        viewModelScope.launch {
            repository
                .upsertConfig(
                    UpsertMilaConfigBody(
                        chatApiBase = state.apiBase,
                        embeddingApiBase = state.embeddingApiBase.ifBlank { state.apiBase },
                        chatModel = state.chatModel,
                        embeddingModel = state.embeddingModel,
                        embeddingDim = state.embeddingDim,
                        modelContextWindow = state.modelContextWindow,
                        enabled = enabled,
                        chatApiKey = state.apiKey.takeIf { it.isNotBlank() },
                        embeddingApiKey = state.apiKey.takeIf { it.isNotBlank() },
                    ),
                ).onFailure {
                    _uiState.value =
                        _uiState.value.copy(
                            enabled = state.enabled,
                            saveError = UiMessage(Res.string.mila_save_failed),
                        )
                }
        }
    }

    fun save(onResult: (Boolean) -> Unit = {}) {
        val state = _uiState.value
        if (state.apiBase.isBlank() || state.chatModel.isBlank()) {
            _uiState.value = state.copy(saveError = UiMessage(Res.string.mila_required_settings))
            onResult(false)
            return
        }
        val embeddingIdentityChanged =
            state.savedEmbeddingModel.isNotBlank() &&
                (
                    state.embeddingModel != state.savedEmbeddingModel ||
                        state.embeddingDim != state.savedEmbeddingDim
                )
        if (embeddingIdentityChanged && !state.reindexConfirmationRequired) {
            _uiState.value =
                state.copy(
                    saveError = UiMessage(Res.string.mila_rebuild_embeddings_confirm),
                    reindexConfirmationRequired = true,
                )
            onResult(false)
            return
        }
        viewModelScope.launch {
            _uiState.value = _uiState.value.copy(isSaving = true, saveError = null)
            val body =
                UpsertMilaConfigBody(
                    chatApiBase = state.apiBase,
                    embeddingApiBase = state.embeddingApiBase.ifBlank { state.apiBase },
                    chatModel = state.chatModel,
                    embeddingModel = state.embeddingModel,
                    embeddingDim = state.embeddingDim,
                    modelContextWindow = state.modelContextWindow,
                    enabled = state.enabled,
                    chatApiKey = state.apiKey.takeIf { it.isNotBlank() },
                    embeddingApiKey = state.apiKey.takeIf { it.isNotBlank() },
                    clearChatApiKey = if (state.apiKey.isBlank() && state.hasChatApiKey) false else null,
                    clearEmbeddingApiKey =
                        if (state.apiKey.isBlank() && state.hasEmbeddingApiKey) false else null,
                )
            repository
                .let { settingsRepository ->
                    if (embeddingIdentityChanged) {
                        settingsRepository.reindexConfig(body)
                    } else {
                        settingsRepository.upsertConfig(body)
                    }
                }.onSuccess { updated ->
                    _uiState.value =
                        _uiState.value.copy(
                            isSaving = false,
                            hasApiKey = updated.hasChatApiKey || updated.hasEmbeddingApiKey,
                            hasChatApiKey = updated.hasChatApiKey,
                            hasEmbeddingApiKey = updated.hasEmbeddingApiKey,
                            embeddingApiBase = updated.embeddingApiBase,
                            apiKey = "",
                            embeddingModel = updated.embeddingModel,
                            embeddingDim = updated.embeddingDim,
                            modelContextWindow = updated.modelContextWindow,
                            savedEmbeddingModel = updated.embeddingModel,
                            savedEmbeddingDim = updated.embeddingDim,
                            reindexConfirmationRequired = false,
                        )
                    onResult(true)
                }.onFailure {
                    _uiState.value =
                        _uiState.value.copy(
                            isSaving = false,
                            saveError = UiMessage(Res.string.mila_save_failed),
                        )
                    onResult(false)
                }
        }
    }

    fun deletePreset(presetId: String) {
        viewModelScope.launch {
            repository
                .deletePromptPreset(presetId)
                .onSuccess {
                    val updated = _uiState.value.presets.filter { it.id != presetId }
                    _uiState.value = _uiState.value.copy(presets = updated)
                }.onFailure {
                    _uiState.value =
                        _uiState.value.copy(saveError = UiMessage(Res.string.mila_preset_delete_failed))
                }
        }
    }

    fun reloadPresets() {
        viewModelScope.launch {
            repository
                .getPromptPresets()
                .onSuccess { response ->
                    val flat = response.groups.flatMap { it.presets }
                    _uiState.value = _uiState.value.copy(presets = flat)
                }.onFailure {
                    _uiState.value =
                        _uiState.value.copy(saveError = UiMessage(Res.string.mila_presets_load_failed))
                }
        }
    }

    fun testConnection() {
        val state = _uiState.value
        viewModelScope.launch {
            _uiState.value =
                _uiState.value.copy(
                    isTesting = true,
                    testResult = null,
                    testError = null,
                )
            repository
                .testConfig(
                    TestMilaConfigBody(
                        chatApiBase = state.apiBase,
                        embeddingApiBase = state.embeddingApiBase.ifBlank { state.apiBase },
                        chatModel = state.chatModel,
                        embeddingModel = state.embeddingModel,
                        embeddingDim = state.embeddingDim,
                        chatApiKey = state.apiKey.takeIf { it.isNotBlank() },
                        embeddingApiKey = state.apiKey.takeIf { it.isNotBlank() },
                    ),
                ).onSuccess { result ->
                    _uiState.value =
                        _uiState.value.copy(
                            isTesting = false,
                            testResult = result,
                            testError = null,
                        )
                }.onFailure {
                    _uiState.value =
                        _uiState.value.copy(
                            isTesting = false,
                            testResult = null,
                            testError = UiMessage(Res.string.mila_connection_failed),
                        )
                }
        }
    }
}
