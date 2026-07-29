package app.indelible.auth.viewmodel

data class LoginState(
    val email: String = "",
    val password: String = "",
    val emailError: String? = null,
    val passwordError: String? = null,
    val serverError: String? = null,
    val isLoading: Boolean = false,
) {
    fun validate(): LoginState {
        val emailErr = validateEmail(email)
        val passwordErr = if (password.isBlank()) "Password is required" else null
        return copy(emailError = emailErr, passwordError = passwordErr, serverError = null)
    }

    val isValid: Boolean
        get() = validateEmail(email) == null && password.isNotBlank()

    companion object {
        private val EMAIL_PATTERN =
            Regex(
                "^[A-Za-z0-9+_.-]+@[A-Za-z0-9.-]+\\.[A-Za-z]{2,}$",
            )

        fun validateEmail(email: String): String? =
            when {
                email.isBlank() -> "Email is required"
                !EMAIL_PATTERN.matches(email) -> "Invalid email format"
                else -> null
            }
    }
}
