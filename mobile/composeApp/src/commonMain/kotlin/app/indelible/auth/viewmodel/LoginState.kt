package app.indelible.auth.viewmodel

import app.indelible.core.i18n.UiMessage
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.auth_email_invalid
import indelible.composeapp.generated.resources.auth_email_required
import indelible.composeapp.generated.resources.auth_password_required

data class LoginState(
    val email: String = "",
    val password: String = "",
    val emailError: UiMessage? = null,
    val passwordError: UiMessage? = null,
    val serverError: UiMessage? = null,
    val isLoading: Boolean = false,
) {
    fun validate(): LoginState {
        val emailErr = validateEmail(email)
        val passwordErr = if (password.isBlank()) UiMessage(Res.string.auth_password_required) else null
        return copy(emailError = emailErr, passwordError = passwordErr, serverError = null)
    }

    val isValid: Boolean
        get() = validateEmail(email) == null && password.isNotBlank()

    companion object {
        private val EMAIL_PATTERN =
            Regex(
                "^[A-Za-z0-9+_.-]+@[A-Za-z0-9.-]+\\.[A-Za-z]{2,}$",
            )

        fun validateEmail(email: String): UiMessage? =
            when {
                email.isBlank() -> UiMessage(Res.string.auth_email_required)
                !EMAIL_PATTERN.matches(email) -> UiMessage(Res.string.auth_email_invalid)
                else -> null
            }
    }
}
