package app.indelible.profile.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import app.indelible.core.i18n.UiMessage
import app.indelible.core.network.ApiException
import app.indelible.profile.repository.AccountRepository
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.profile_password_change_failed
import indelible.composeapp.generated.resources.profile_password_changed
import indelible.composeapp.generated.resources.profile_password_incorrect
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

sealed class ChangePasswordEffect {
    data class ShowSnackbar(
        val message: UiMessage,
    ) : ChangePasswordEffect()

    data object NavigateBack : ChangePasswordEffect()
}

private const val HTTP_UNAUTHORIZED = 401

class ChangePasswordViewModel(
    private val repository: AccountRepository,
) : ViewModel() {
    private val _isLoading = MutableStateFlow(false)
    val isLoading: StateFlow<Boolean> = _isLoading.asStateFlow()

    private val _effects = MutableSharedFlow<ChangePasswordEffect>()
    val effects: SharedFlow<ChangePasswordEffect> = _effects.asSharedFlow()

    fun changePassword(
        currentPassword: String,
        newPassword: String,
    ) {
        _isLoading.value = true
        viewModelScope.launch {
            repository
                .changePassword(currentPassword, newPassword)
                .onSuccess {
                    _isLoading.value = false
                    _effects.emit(
                        ChangePasswordEffect.ShowSnackbar(UiMessage(Res.string.profile_password_changed)),
                    )
                    _effects.emit(ChangePasswordEffect.NavigateBack)
                }.onFailure { error ->
                    _isLoading.value = false
                    val message =
                        when {
                            error is ApiException && error.statusCode == HTTP_UNAUTHORIZED ->
                                UiMessage(Res.string.profile_password_incorrect)
                            else -> UiMessage(Res.string.profile_password_change_failed)
                        }
                    _effects.emit(ChangePasswordEffect.ShowSnackbar(message))
                }
        }
    }
}
