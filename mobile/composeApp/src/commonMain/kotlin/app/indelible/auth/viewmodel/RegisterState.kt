package app.indelible.auth.viewmodel

data class RegisterState(
    val displayName: String = "",
    val email: String = "",
    val password: String = "",
    val confirmPassword: String = "",
    val displayNameError: String? = null,
    val emailError: String? = null,
    val passwordError: String? = null,
    val confirmPasswordError: String? = null,
    val serverError: String? = null,
    val isLoading: Boolean = false,
) {
    fun validate(): RegisterState {
        val nameErr = if (displayName.isBlank()) "Display name is required" else null
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

        fun validatePassword(password: String): String? =
            when {
                password.isBlank() -> "Password is required"
                password.length < MIN_PASSWORD_LENGTH ->
                    "Password must be at least $MIN_PASSWORD_LENGTH characters"
                else -> null
            }

        fun validateConfirmPassword(
            password: String,
            confirmPassword: String,
        ): String? =
            when {
                confirmPassword.isBlank() -> "Please confirm your password"
                confirmPassword != password -> "Passwords do not match"
                else -> null
            }
    }
}
