package app.indelible.auth.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import app.indelible.auth.server.ServerHealthChecker
import app.indelible.auth.server.ServerUrlForm
import app.indelible.auth.server.ServerUrlValidation
import app.indelible.core.i18n.UiMessage
import app.indelible.core.storage.TokenStorage
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.auth_server_unreachable
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

data class ConnectServerState(
    val url: String = "",
    val isChecking: Boolean = false,
    val error: UiMessage? = null,
    val pendingCleartextUrl: String? = null,
)

sealed interface ServerSetupState {
    data object Unknown : ServerSetupState

    data object Required : ServerSetupState

    data class Configured(
        val serverUrl: String,
    ) : ServerSetupState
}

class ConnectServerViewModel(
    private val tokenStorage: TokenStorage,
    private val healthChecker: ServerHealthChecker,
    private val bakedDefaultUrl: String,
    private val devPrefillUrl: String,
) : ViewModel() {
    private val _setupState = MutableStateFlow<ServerSetupState>(ServerSetupState.Unknown)
    val setupState: StateFlow<ServerSetupState> = _setupState.asStateFlow()

    private val _state = MutableStateFlow(ConnectServerState())
    val state: StateFlow<ConnectServerState> = _state.asStateFlow()

    private val _connectedUrl = MutableStateFlow<String?>(null)
    val connectedUrl: StateFlow<String?> = _connectedUrl.asStateFlow()

    init {
        viewModelScope.launch {
            val stored = tokenStorage.getServerUrl()
            val baked = bakedDefaultUrl.trim().ifEmpty { null }
            _setupState.value =
                when {
                    stored != null -> ServerSetupState.Configured(stored)
                    baked != null -> ServerSetupState.Configured(baked)
                    else -> ServerSetupState.Required
                }
            val known = stored ?: baked ?: devPrefillUrl.trim().ifEmpty { null }
            _state.value = _state.value.copy(url = known.orEmpty())
        }
    }

    fun updateUrl(url: String) {
        _state.value = _state.value.copy(url = url, error = null)
    }

    fun connect() {
        when (val validation = ServerUrlForm.validate(_state.value.url)) {
            is ServerUrlValidation.Invalid -> {
                _state.value = _state.value.copy(error = validation.message)
            }
            is ServerUrlValidation.NeedsCleartextConsent -> {
                _state.value = _state.value.copy(pendingCleartextUrl = validation.url)
            }
            is ServerUrlValidation.Ready -> {
                checkAndPersist(validation.url)
            }
        }
    }

    fun confirmCleartext() {
        val pending = _state.value.pendingCleartextUrl ?: return
        _state.value = _state.value.copy(pendingCleartextUrl = null)
        checkAndPersist(pending)
    }

    fun dismissCleartextWarning() {
        _state.value = _state.value.copy(pendingCleartextUrl = null)
    }

    fun consumeConnectedEvent() {
        _connectedUrl.value = null
    }

    private fun checkAndPersist(url: String) {
        viewModelScope.launch {
            _state.value = _state.value.copy(isChecking = true, error = null)
            healthChecker
                .check(url)
                .onSuccess {
                    tokenStorage.saveServerUrl(url)
                    _setupState.value = ServerSetupState.Configured(url)
                    _state.value = _state.value.copy(isChecking = false, url = url)
                    _connectedUrl.value = url
                }.onFailure {
                    _state.value =
                        _state.value.copy(
                            isChecking = false,
                            error = UiMessage(Res.string.auth_server_unreachable),
                        )
                }
        }
    }
}
