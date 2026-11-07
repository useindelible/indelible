package app.indelible.auth.viewmodel

import app.indelible.auth.repository.ApiAuthRepository
import app.indelible.core.network.ApiClient
import app.indelible.core.offline.testSessionTransitions
import app.indelible.core.storage.TokenStorage

private const val FAR_FUTURE_EXPIRY = 4_102_444_800L

internal fun signedInProfileJson() =
    """
    {
        "id": "usr_01ABCDEF",
        "object": "user",
        "email": "user@example.com",
        "display_name": "Test User",
        "email_verified": true,
        "onboarding_completed": true,
        "has_password": true,
        "locale": "en",
        "theme": "auto",
        "timezone": "UTC",
        "created_at": "2024-01-01T00:00:00Z",
        "updated_at": "2024-01-01T00:00:00Z"
    }
    """.trimIndent()

internal fun testAuthViewModel(
    apiClient: ApiClient,
    tokenStorage: TokenStorage,
): AuthViewModel =
    AuthViewModel(
        ApiAuthRepository(apiClient.authApiService, apiClient.accountApiService),
        tokenStorage,
        testOfflineAccount(tokenStorage),
        testSessionTransitions(tokenStorage),
    )

internal fun signedInAuthJson(
    accessToken: String,
    refreshToken: String,
    displayName: String = "Test User",
    emailVerified: Boolean = true,
    onboardingCompleted: Boolean = true,
) = """
    {
        "id": "usr_01ABCDEF",
        "object": "user",
        "email": "user@example.com",
        "display_name": "$displayName",
        "email_verified": $emailVerified,
        "onboarding_completed": $onboardingCompleted,
        "access_token": "$accessToken",
        "refresh_token": "$refreshToken",
        "expires_at": $FAR_FUTURE_EXPIRY
    }
    """.trimIndent()
