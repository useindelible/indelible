package app.indelible.auth.viewmodel

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

@OptIn(ExperimentalCoroutinesApi::class)
class AuthViewModelRegisterTest {
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
    fun registerSuccessTransitionsToNeedsVerificationAndSavesTokens() =
        runTest {
            val tokenStorage = InMemoryTokenStorage()
            val engine =
                MockEngine { request ->
                    when (request.url.encodedPath) {
                        "/api/v1/auth/register" ->
                            respond(
                                content =
                                    signedInAuthJson(
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
            val viewModel = testAuthViewModel(apiClient, tokenStorage)
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
            val viewModel = testAuthViewModel(apiClient, tokenStorage)
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
                                    signedInAuthJson(
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
            val viewModel = testAuthViewModel(apiClient, tokenStorage)
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
}
