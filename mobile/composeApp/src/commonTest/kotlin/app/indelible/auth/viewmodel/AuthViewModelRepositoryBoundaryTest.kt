package app.indelible.auth.viewmodel

import app.indelible.api.generated.models.AuthResponse
import app.indelible.api.generated.models.OAuthProvidersResponse
import app.indelible.auth.repository.AuthRepository
import app.indelible.core.model.AuthUser
import app.indelible.core.network.NativeOAuthTokenResponse
import app.indelible.core.storage.InMemoryTokenStorage
import kotlin.test.Test
import kotlin.test.assertNotNull

class AuthViewModelRepositoryBoundaryTest {
    @Test
    fun authViewModelIsConstructedFromRepository() {
        val viewModel = AuthViewModel(FakeAuthRepository(), InMemoryTokenStorage())

        assertNotNull(viewModel)
    }
}

private class FakeAuthRepository : AuthRepository {
    override suspend fun login(
        email: String,
        password: String,
    ): Result<AuthResponse> = unused()

    override suspend fun register(
        name: String,
        email: String,
        password: String,
    ): Result<AuthResponse> = unused()

    override suspend fun forgotPassword(email: String): Result<Unit> = Result.success(Unit)

    override suspend fun logout(): Result<Unit> = Result.success(Unit)

    override suspend fun getSession(): Result<AuthUser> = unused()

    override suspend fun resendVerification(): Result<Unit> = Result.success(Unit)

    override suspend fun getOAuthProviders(): Result<OAuthProvidersResponse> = unused()

    override suspend fun nativeOAuthStartUrl(
        providerId: String,
        codeChallenge: String,
        appState: String,
    ): String = "https://example.com"

    override suspend fun exchangeNativeOAuthCode(
        code: String,
        codeVerifier: String,
    ): Result<NativeOAuthTokenResponse> = unused()

    override suspend fun updateProfile(
        displayName: String?,
        theme: String?,
    ): Result<AuthUser> = unused()

    override suspend fun fetchAvatarBytes(avatarUrl: String): Result<ByteArray> = unused()
}

private fun <T> unused(): Result<T> = Result.failure(UnsupportedOperationException("not used"))
