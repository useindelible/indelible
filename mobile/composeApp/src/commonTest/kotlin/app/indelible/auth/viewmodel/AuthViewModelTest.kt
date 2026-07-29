package app.indelible.auth.viewmodel

import app.indelible.auth.repository.ApiAuthRepository
import app.indelible.core.network.ApiClient
import app.indelible.core.storage.InMemoryTokenStorage
import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.respond
import io.ktor.http.HttpHeaders
import io.ktor.http.HttpStatusCode
import io.ktor.http.headersOf
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import kotlin.test.AfterTest
import kotlin.test.BeforeTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertIs
import kotlin.test.assertNotNull
import kotlin.test.assertNull

@OptIn(ExperimentalCoroutinesApi::class)
class AuthViewModelTest {
    private val testDispatcher = UnconfinedTestDispatcher()
    private val jsonHeaders = headersOf(HttpHeaders.ContentType, "application/json")

    @BeforeTest
    fun setUp() {
        Dispatchers.setMain(testDispatcher)
    }

    @AfterTest
    fun tearDown() {
        Dispatchers.resetMain()
    }

    @Test
    fun initialStateIsLoading() {
        val lazyDispatcher = StandardTestDispatcher()
        Dispatchers.setMain(lazyDispatcher)
        try {
            val tokenStorage = InMemoryTokenStorage()
            val engine = MockEngine { respond("", HttpStatusCode.Unauthorized) }
            val apiClient = ApiClient(tokenStorage, engine = engine)
            val viewModel = AuthViewModel(ApiAuthRepository(apiClient.authApiService, apiClient.accountApiService), tokenStorage)

            assertIs<AuthState.Loading>(viewModel.authState.value)
        } finally {
            Dispatchers.resetMain()
            Dispatchers.setMain(testDispatcher)
        }
    }

    @Test
    fun unauthenticatedWhenNoStoredAuth() =
        runTest {
            val tokenStorage = InMemoryTokenStorage()
            val apiClient = ApiClient(tokenStorage, engine = MockEngine { respond("", HttpStatusCode.OK) })
            val viewModel = AuthViewModel(ApiAuthRepository(apiClient.authApiService, apiClient.accountApiService), tokenStorage)

            // Await the terminal init state deterministically instead of pinning the
            // test to the shared Main dispatcher via runTest(testDispatcher), which
            // conflicts with the setMain/resetMain in setUp/tearDown and made test
            // ordering matter. Mirrors the await pattern used by the other tests here.
            viewModel.authState.first { it is AuthState.Unauthenticated }

            assertIs<AuthState.Unauthenticated>(viewModel.authState.value)
        }

    @Test
    fun authenticatedAfterSuccessfulLoginSavesAllTokens() =
        runTest {
            val tokenStorage = InMemoryTokenStorage()
            val engine =
                MockEngine { request ->
                    when (request.url.encodedPath) {
                        "/api/v1/auth/login" ->
                            respond(
                                content =
                                    authResponseJson(
                                        accessToken = "test-token-123",
                                        refreshToken = "test-refresh-123",
                                    ),
                                status = HttpStatusCode.OK,
                                headers = jsonHeaders,
                            )
                        else -> respond("", HttpStatusCode.Unauthorized)
                    }
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val viewModel = AuthViewModel(ApiAuthRepository(apiClient.authApiService, apiClient.accountApiService), tokenStorage)
            viewModel.authState.first { it is AuthState.Unauthenticated }

            viewModel.updateLoginEmail("user@example.com")
            viewModel.updateLoginPassword("password123")
            viewModel.login()
            viewModel.authState.first { it is AuthState.Authenticated }

            val user = (viewModel.authState.value as AuthState.Authenticated).user
            assertEquals("Test User", user.displayName)
            assertEquals("test-token-123", tokenStorage.getToken())
            assertEquals("test-refresh-123", tokenStorage.getRefreshToken())
            assertEquals(FAR_FUTURE_EXPIRY, tokenStorage.getExpiresAt())
        }

    @Test
    fun loginErrorSetsServerError() =
        runTest {
            val tokenStorage = InMemoryTokenStorage()
            val engine =
                MockEngine { request ->
                    when (request.url.encodedPath) {
                        "/api/v1/auth/login" ->
                            respond(
                                content = """{"error":"invalid_credentials","message":"Invalid email or password"}""",
                                status = HttpStatusCode.Unauthorized,
                                headers = jsonHeaders,
                            )
                        else -> respond("", HttpStatusCode.Unauthorized)
                    }
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val viewModel = AuthViewModel(ApiAuthRepository(apiClient.authApiService, apiClient.accountApiService), tokenStorage)
            viewModel.authState.first { it is AuthState.Unauthenticated }

            viewModel.updateLoginEmail("user@example.com")
            viewModel.updateLoginPassword("wrongpassword")
            viewModel.login()
            viewModel.loginState.first { !it.isLoading && it.serverError != null }

            assertIs<AuthState.Unauthenticated>(viewModel.authState.value)
            assertEquals("Invalid email or password", viewModel.loginState.value.serverError)
        }

    @Test
    fun registerSuccessTransitionsToNeedsVerificationAndSavesTokens() =
        runTest {
            val tokenStorage = InMemoryTokenStorage()
            val engine =
                MockEngine { request ->
                    when (request.url.encodedPath) {
                        "/api/v1/auth/register" ->
                            respond(
                                content =
                                    authResponseJson(
                                        accessToken = "reg-token-456",
                                        refreshToken = "reg-refresh-456",
                                        displayName = "New User",
                                        emailVerified = false,
                                        onboardingCompleted = false,
                                    ),
                                status = HttpStatusCode.OK,
                                headers = jsonHeaders,
                            )
                        else -> respond("", HttpStatusCode.Unauthorized)
                    }
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val viewModel = AuthViewModel(ApiAuthRepository(apiClient.authApiService, apiClient.accountApiService), tokenStorage)
            viewModel.authState.first { it is AuthState.Unauthenticated }

            viewModel.updateRegisterDisplayName("New User")
            viewModel.updateRegisterEmail("new@example.com")
            viewModel.updateRegisterPassword("password123")
            viewModel.updateRegisterConfirmPassword("password123")
            viewModel.register()
            viewModel.authState.first { it is AuthState.NeedsVerification }

            val user = (viewModel.authState.value as AuthState.NeedsVerification).user
            assertEquals("New User", user.displayName)
            assertEquals("reg-token-456", tokenStorage.getToken())
            assertEquals("reg-refresh-456", tokenStorage.getRefreshToken())
        }

    @Test
    fun oauthProviderFailureFailsClosedForSignupState() =
        runTest {
            val tokenStorage = InMemoryTokenStorage()
            val engine =
                MockEngine { request ->
                    when (request.url.encodedPath) {
                        "/api/v1/auth/providers" ->
                            respond(
                                content = """{"error":"server_error","message":"offline"}""",
                                status = HttpStatusCode.InternalServerError,
                                headers = jsonHeaders,
                            )
                        else -> respond("", HttpStatusCode.Unauthorized)
                    }
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val viewModel = AuthViewModel(ApiAuthRepository(apiClient.authApiService, apiClient.accountApiService), tokenStorage)
            advanceUntilIdle()

            assertFalse(viewModel.signupsEnabled.value)
            assertFalse(viewModel.setupRequired.value)
        }

    @Test
    fun registerSuccessRefreshesFirstRunSignupState() =
        runTest {
            val tokenStorage = InMemoryTokenStorage()
            var providerRequests = 0
            val engine =
                MockEngine { request ->
                    when (request.url.encodedPath) {
                        "/api/v1/auth/providers" -> {
                            providerRequests += 1
                            respond(
                                content =
                                    oauthProvidersJson(
                                        signupsEnabled = providerRequests == 1,
                                        setupRequired = providerRequests == 1,
                                    ),
                                status = HttpStatusCode.OK,
                                headers = jsonHeaders,
                            )
                        }
                        "/api/v1/auth/register" ->
                            respond(
                                content =
                                    authResponseJson(
                                        accessToken = "reg-token-setup",
                                        refreshToken = "reg-refresh-setup",
                                        displayName = "First Owner",
                                        emailVerified = false,
                                        onboardingCompleted = false,
                                    ),
                                status = HttpStatusCode.OK,
                                headers = jsonHeaders,
                            )
                        else -> respond("", HttpStatusCode.Unauthorized)
                    }
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val viewModel = AuthViewModel(ApiAuthRepository(apiClient.authApiService, apiClient.accountApiService), tokenStorage)
            viewModel.setupRequired.first { it }

            viewModel.updateRegisterDisplayName("First Owner")
            viewModel.updateRegisterEmail("owner@example.com")
            viewModel.updateRegisterPassword("password123")
            viewModel.updateRegisterConfirmPassword("password123")
            viewModel.register()
            viewModel.authState.first { it is AuthState.NeedsVerification }
            viewModel.setupRequired.first { !it }

            assertFalse(viewModel.signupsEnabled.value)
            assertFalse(viewModel.setupRequired.value)
        }

    @Test
    fun initializeWithRefreshTokenOnlyRestoresSession() =
        runTest {
            val tokenStorage = InMemoryTokenStorage()
            tokenStorage.saveRefreshToken("refresh-only")

            val engine =
                MockEngine { request ->
                    when (request.url.encodedPath) {
                        "/api/v1/auth/refresh" ->
                            respond(
                                content =
                                    refreshResponseJson(
                                        accessToken = "access-restored",
                                        refreshToken = "refresh-restored",
                                    ),
                                status = HttpStatusCode.OK,
                                headers = jsonHeaders,
                            )
                        "/api/v1/me" ->
                            respond(
                                content = profileResponseJson(),
                                status = HttpStatusCode.OK,
                                headers = jsonHeaders,
                            )
                        else -> respond("", HttpStatusCode.NotFound)
                    }
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val viewModel = AuthViewModel(ApiAuthRepository(apiClient.authApiService, apiClient.accountApiService), tokenStorage)

            viewModel.authState.first { it is AuthState.Authenticated }

            assertEquals("access-restored", tokenStorage.getToken())
            assertEquals("refresh-restored", tokenStorage.getRefreshToken())
        }

    @Test
    fun logoutClearsAllStoredAuth() =
        runTest {
            val tokenStorage = InMemoryTokenStorage()
            tokenStorage.saveToken("existing-token")
            tokenStorage.saveRefreshToken("existing-refresh")
            tokenStorage.saveExpiresAt(FAR_FUTURE_EXPIRY)
            tokenStorage.savePendingItems("""[{"id":"pending-1"}]""")

            val engine =
                MockEngine { request ->
                    when (request.url.encodedPath) {
                        "/api/v1/me" ->
                            respond(
                                content = profileResponseJson(),
                                status = HttpStatusCode.OK,
                                headers = jsonHeaders,
                            )
                        "/api/v1/auth/logout" -> respond("", HttpStatusCode.NoContent, jsonHeaders)
                        else -> respond("", HttpStatusCode.OK)
                    }
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val viewModel = AuthViewModel(ApiAuthRepository(apiClient.authApiService, apiClient.accountApiService), tokenStorage)
            viewModel.authState.first { it is AuthState.Authenticated }

            viewModel.logout()
            viewModel.authState.first { it is AuthState.Unauthenticated }

            assertNull(tokenStorage.getToken())
            assertNull(tokenStorage.getRefreshToken())
            assertNull(tokenStorage.getExpiresAt())
            assertNull(tokenStorage.getPendingItems())
        }

    @Test
    fun logoutKeepsLocalWipeButReportsRevocationFailure() =
        runTest {
            val tokenStorage = InMemoryTokenStorage()
            tokenStorage.saveToken("existing-token")
            tokenStorage.saveRefreshToken("existing-refresh")
            tokenStorage.saveExpiresAt(FAR_FUTURE_EXPIRY)

            val engine =
                MockEngine { request ->
                    when (request.url.encodedPath) {
                        "/api/v1/me" ->
                            respond(
                                content = profileResponseJson(),
                                status = HttpStatusCode.OK,
                                headers = jsonHeaders,
                            )
                        "/api/v1/auth/logout" ->
                            respond(
                                content = """{"error":"server_error","message":"offline"}""",
                                status = HttpStatusCode.InternalServerError,
                                headers = jsonHeaders,
                            )
                        else -> respond("", HttpStatusCode.OK)
                    }
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val viewModel = AuthViewModel(ApiAuthRepository(apiClient.authApiService, apiClient.accountApiService), tokenStorage)
            viewModel.authState.first { it is AuthState.Authenticated }

            viewModel.logout()
            viewModel.authState.first { it is AuthState.Unauthenticated }

            assertNull(tokenStorage.getToken())
            assertNull(tokenStorage.getRefreshToken())
            assertNull(tokenStorage.getExpiresAt())
            assertNotNull(viewModel.loginState.value.serverError)
        }

    @Test
    fun forceLogoutClearsWithoutServerCall() =
        runTest {
            val tokenStorage = InMemoryTokenStorage()
            tokenStorage.saveToken("existing-token")
            tokenStorage.saveRefreshToken("existing-refresh")
            tokenStorage.saveExpiresAt(FAR_FUTURE_EXPIRY)

            val apiClient = ApiClient(tokenStorage, engine = MockEngine { respond("", HttpStatusCode.OK) })
            val viewModel = AuthViewModel(ApiAuthRepository(apiClient.authApiService, apiClient.accountApiService), tokenStorage)

            viewModel.forceLogout()
            advanceUntilIdle()

            assertIs<AuthState.Unauthenticated>(viewModel.authState.value)
            assertNull(tokenStorage.getToken())
            assertNull(tokenStorage.getRefreshToken())
        }

    @Test
    fun loginValidationPreventsApiCall() =
        runTest {
            val tokenStorage = InMemoryTokenStorage()
            var loginRequests = 0
            val engine =
                MockEngine { request ->
                    if (request.url.encodedPath == "/api/v1/auth/login") loginRequests++
                    respond("", HttpStatusCode.Unauthorized)
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val viewModel = AuthViewModel(ApiAuthRepository(apiClient.authApiService, apiClient.accountApiService), tokenStorage)
            // Await init settling deterministically; counting only login-path requests
            // keeps this immune to the unconditional OAuth-providers fetch on construction.
            viewModel.authState.first { it is AuthState.Unauthenticated }

            viewModel.updateLoginEmail("")
            viewModel.updateLoginPassword("")
            viewModel.login()
            advanceUntilIdle()

            assertEquals(0, loginRequests)
            assertEquals("Email is required", viewModel.loginState.value.emailError)
            assertEquals("Password is required", viewModel.loginState.value.passwordError)
        }

    private fun authResponseJson(
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

    private fun refreshResponseJson(
        accessToken: String,
        refreshToken: String,
    ) = """
        {
            "access_token": "$accessToken",
            "refresh_token": "$refreshToken",
            "expires_at": $FAR_FUTURE_EXPIRY
        }
        """.trimIndent()

    private fun profileResponseJson() =
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

    private fun oauthProvidersJson(
        signupsEnabled: Boolean,
        setupRequired: Boolean,
    ) = """
        {
            "providers": [],
            "signups_enabled": $signupsEnabled,
            "setup_required": $setupRequired
        }
        """.trimIndent()

    companion object {
        private const val FAR_FUTURE_EXPIRY = 4_102_444_800L
    }
}
