package app.indelible.auth.viewmodel

import app.indelible.core.i18n.UiMessage

data class ForgotPasswordState(
    val email: String = "",
    val emailError: UiMessage? = null,
    val serverError: UiMessage? = null,
    val isLoading: Boolean = false,
    val isSubmitted: Boolean = false,
)
