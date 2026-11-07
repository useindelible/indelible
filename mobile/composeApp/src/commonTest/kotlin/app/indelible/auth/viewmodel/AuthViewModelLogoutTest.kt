package app.indelible.auth.viewmodel

import app.indelible.core.i18n.UiMessage
import app.indelible.core.network.ApiClient
import app.indelible.core.storage.InMemoryTokenStorage
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.auth_logout_revoke_failed
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
import kotlin.test.assertIs
import kotlin.test.assertNull

@OptIn(ExperimentalCoroutinesApi::class)
class AuthViewModelLogoutTest {
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
                                content = signedInProfileJson(),
                                status = HttpStatusCode.OK,
                                headers = jsonHeaders,
                            )
                        "/api/v1/auth/logout" -> respond("", HttpStatusCode.NoContent, jsonHeaders)
                        else -> respond("", HttpStatusCode.OK)
                    }
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val viewModel = testAuthViewModel(apiClient, tokenStorage)
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
                                content = signedInProfileJson(),
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
            val viewModel = testAuthViewModel(apiClient, tokenStorage)
            viewModel.authState.first { it is AuthState.Authenticated }

            viewModel.logout()
            viewModel.authState.first { it is AuthState.Unauthenticated }

            assertNull(tokenStorage.getToken())
            assertNull(tokenStorage.getRefreshToken())
            assertNull(tokenStorage.getExpiresAt())
            assertEquals(
                UiMessage(Res.string.auth_logout_revoke_failed),
                viewModel.loginState.first { it.serverError != null }.serverError,
            )
        }

    @Test
    fun forceLogoutClearsWithoutServerCall() =
        runTest {
            val tokenStorage = InMemoryTokenStorage()
            tokenStorage.saveToken("existing-token")
            tokenStorage.saveRefreshToken("existing-refresh")
            tokenStorage.saveExpiresAt(FAR_FUTURE_EXPIRY)

            val apiClient = ApiClient(tokenStorage, engine = MockEngine { respond("", HttpStatusCode.OK) })
            val viewModel = testAuthViewModel(apiClient, tokenStorage)

            viewModel.forceLogout()
            advanceUntilIdle()

            assertIs<AuthState.Unauthenticated>(viewModel.authState.value)
            assertNull(tokenStorage.getToken())
            assertNull(tokenStorage.getRefreshToken())
        }

    companion object {
        private const val FAR_FUTURE_EXPIRY = 4_102_444_800L
    }
}
