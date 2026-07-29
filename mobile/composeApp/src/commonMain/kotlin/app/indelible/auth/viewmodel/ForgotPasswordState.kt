package app.indelible.auth.viewmodel

data class ForgotPasswordState(
    val email: String = "",
    val emailError: String? = null,
    val serverError: String? = null,
    val isLoading: Boolean = false,
    val isSubmitted: Boolean = false,
)
