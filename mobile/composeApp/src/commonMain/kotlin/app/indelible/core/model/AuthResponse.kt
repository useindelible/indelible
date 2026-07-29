package app.indelible.core.model

import kotlinx.serialization.Serializable

typealias AuthResponse = app.indelible.api.generated.models.AuthResponse
typealias RefreshResponse = app.indelible.api.generated.models.RefreshResponse
typealias LoginRequest = app.indelible.api.generated.models.LoginRequest
typealias RegisterRequest = app.indelible.api.generated.models.RegisterRequest
typealias ForgotPasswordRequest = app.indelible.api.generated.models.ForgotPasswordRequest
typealias ProfileResponse = app.indelible.api.generated.models.ProfileResponse

fun app.indelible.api.generated.models.AuthResponse.toAuthUser(): AuthUser =
    AuthUser(
        id = id,
        email = email,
        displayName = displayName,
        emailVerified = emailVerified,
        onboardingCompleted = onboardingCompleted,
    )

fun app.indelible.api.generated.models.ProfileResponse.toAuthUser(): AuthUser =
    AuthUser(
        id = id,
        email = email,
        displayName = displayName,
        emailVerified = emailVerified,
        onboardingCompleted = onboardingCompleted,
        ingestEmail = ingestEmail,
        ingestLibraryEmail = ingestLibraryEmail,
        avatarUrl = avatarUrl,
    )

@Serializable
data class ApiError(
    val error: String,
    val message: String? = null,
)
