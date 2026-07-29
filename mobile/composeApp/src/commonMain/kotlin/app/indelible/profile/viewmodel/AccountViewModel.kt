package app.indelible.profile.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import app.indelible.profile.repository.AccountRepository
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.launch

sealed class AccountEffect {
    data class ShowSnackbar(
        val message: String,
    ) : AccountEffect()

    data object AccountDeleted : AccountEffect()
}

class AccountViewModel(
    private val repository: AccountRepository,
) : ViewModel() {
    private val _effects = MutableSharedFlow<AccountEffect>()
    val effects: SharedFlow<AccountEffect> = _effects.asSharedFlow()

    fun deleteAccount(confirmation: String) {
        viewModelScope.launch {
            repository
                .deleteAccount(confirmation)
                .onSuccess { _effects.emit(AccountEffect.AccountDeleted) }
                .onFailure { error ->
                    _effects.emit(AccountEffect.ShowSnackbar(error.message ?: "Failed to delete account"))
                }
        }
    }
}
