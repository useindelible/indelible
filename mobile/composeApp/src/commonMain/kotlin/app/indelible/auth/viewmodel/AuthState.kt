package app.indelible.auth.viewmodel

import app.indelible.core.model.AuthUser

sealed class AuthState {
    data object Loading : AuthState()

    data object Unauthenticated : AuthState()

    data class Authenticated(
        val user: AuthUser,
    ) : AuthState()

    data class NeedsVerification(
        val user: AuthUser,
    ) : AuthState()

    data class NeedsOnboarding(
        val user: AuthUser,
    ) : AuthState()
}
