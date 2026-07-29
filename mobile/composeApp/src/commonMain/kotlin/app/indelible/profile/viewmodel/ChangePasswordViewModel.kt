package app.indelible.profile.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import app.indelible.core.network.ApiException
import app.indelible.profile.repository.AccountRepository
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

sealed class ChangePasswordEffect {
    data class ShowSnackbar(
        val message: String,
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
                    _effects.emit(ChangePasswordEffect.ShowSnackbar("Password changed successfully"))
                    _effects.emit(ChangePasswordEffect.NavigateBack)
                }.onFailure { error ->
                    _isLoading.value = false
                    val message =
                        when {
                            error is ApiException && error.statusCode == HTTP_UNAUTHORIZED ->
                                "Current password is incorrect"
                            else -> error.message ?: "Failed to change password"
                        }
                    _effects.emit(ChangePasswordEffect.ShowSnackbar(message))
                }
        }
    }
}
