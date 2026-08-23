package app.indelible.auth.viewmodel

import app.indelible.core.i18n.UiMessage
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.auth_confirm_password_required
import indelible.composeapp.generated.resources.auth_display_name_required
import indelible.composeapp.generated.resources.auth_password_min_length
import indelible.composeapp.generated.resources.auth_password_required
import indelible.composeapp.generated.resources.auth_passwords_mismatch

data class RegisterState(
    val displayName: String = "",
    val email: String = "",
    val password: String = "",
    val confirmPassword: String = "",
    val displayNameError: UiMessage? = null,
    val emailError: UiMessage? = null,
    val passwordError: UiMessage? = null,
    val confirmPasswordError: UiMessage? = null,
    val serverError: UiMessage? = null,
    val isLoading: Boolean = false,
) {
    fun validate(): RegisterState {
        val nameErr = if (displayName.isBlank()) UiMessage(Res.string.auth_display_name_required) else null
        val emailErr = LoginState.validateEmail(email)
        val passErr = validatePassword(password)
        val confirmErr = validateConfirmPassword(password, confirmPassword)
        return copy(
            displayNameError = nameErr,
            emailError = emailErr,
            passwordError = passErr,
            confirmPasswordError = confirmErr,
            serverError = null,
        )
    }

    val isValid: Boolean
        get() =
            displayName.isNotBlank() &&
                LoginState.validateEmail(email) == null &&
                validatePassword(password) == null &&
                validateConfirmPassword(password, confirmPassword) == null

    companion object {
        private const val MIN_PASSWORD_LENGTH = 8

        fun validatePassword(password: String): UiMessage? =
            when {
                password.isBlank() -> UiMessage(Res.string.auth_password_required)
                password.length < MIN_PASSWORD_LENGTH ->
                    UiMessage(Res.string.auth_password_min_length, listOf(MIN_PASSWORD_LENGTH))
                else -> null
            }

        fun validateConfirmPassword(
            password: String,
            confirmPassword: String,
        ): UiMessage? =
            when {
                confirmPassword.isBlank() -> UiMessage(Res.string.auth_confirm_password_required)
                confirmPassword != password -> UiMessage(Res.string.auth_passwords_mismatch)
                else -> null
            }
    }
}
