package app.indelible.auth.viewmodel

import app.indelible.auth.repository.ApiAuthRepository
import app.indelible.core.network.ApiClient
import app.indelible.core.network.normalizedOrigin
import app.indelible.core.offline.CachedDocumentRow
import app.indelible.core.offline.OfflineStore
import app.indelible.core.offline.ScopePurger
import app.indelible.core.offline.SqlDelightOfflineStore
import app.indelible.core.offline.currentOfflineScope
import app.indelible.core.offline.testOutboxWorker
import app.indelible.core.offline.testSessionTransitions
import app.indelible.core.storage.InMemoryTokenStorage
import app.indelible.db.testOfflineDatabase
import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.respond
import io.ktor.http.HttpHeaders
import io.ktor.http.HttpStatusCode
import io.ktor.http.headersOf
import kotlinx.coroutines.CompletableDeferred
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
import kotlin.test.assertNotNull
import kotlin.test.assertNull
import kotlin.test.assertTrue

@OptIn(ExperimentalCoroutinesApi::class)
class AuthViewModelPurgeTest {
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
    fun forceLogoutDoesNotPurgeTheScope() =
        runTest {
            val tokenStorage = InMemoryTokenStorage()
            tokenStorage.saveServerUrl("https://example.com")
            tokenStorage.saveUserId("usr_ABC")
            tokenStorage.saveToken("existing-token")
            val scope = tokenStorage.currentOfflineScope()!!
            val store = SqlDelightOfflineStore(testOfflineDatabase())
            store.clientIdentity(scope)
            store.upsertCachedDocument(scope, cachedDocumentRow())

            val apiClient = ApiClient(tokenStorage, engine = MockEngine { respond("", HttpStatusCode.OK) })
            val viewModel = authViewModel(apiClient, tokenStorage, ScopePurger(store))

            viewModel.forceLogout(0)
            advanceUntilIdle()

            assertIs<AuthState.Unauthenticated>(viewModel.authState.value)
            assertNotNull(store.cachedDocument(scope, "doc_1"))
            assertEquals(listOf(scope to false), store.scopesWithState())
        }

    @Test
    fun logoutSignsOutEvenWhenThePurgeThrows() =
        runTest {
            val tokenStorage = signedInTokenStorage()
            val store = SqlDelightOfflineStore(testOfflineDatabase())
            val apiClient = ApiClient(tokenStorage, engine = sessionEngine())
            val attempted = CompletableDeferred<Unit>()
            val viewModel =
                authViewModel(apiClient, tokenStorage, ScopePurger(ThrowingPurgeRowsStore(store, attempted)))
            viewModel.authState.first { it is AuthState.Authenticated }

            viewModel.logout()
            viewModel.authState.first { it is AuthState.Unauthenticated }
            attempted.await()
            advanceUntilIdle()

            assertIs<AuthState.Unauthenticated>(viewModel.authState.value)
            assertNull(tokenStorage.getToken())
            assertNull(tokenStorage.getUserId())
        }

    @Test
    fun logoutPurgesTheActiveScopeAfterClearingTokens() =
        runTest {
            val tokenStorage = signedInTokenStorage()
            val store = SqlDelightOfflineStore(testOfflineDatabase())
            val apiClient = ApiClient(tokenStorage, engine = sessionEngine())
            val purged = CompletableDeferred<Unit>()
            val purgedScopes = mutableListOf<String>()
            val tokenAtPurge = mutableListOf<String?>()
            val purger =
                ScopePurger(PurgeFinishedStore(store, purged)) { scope ->
                    purgedScopes += scope
                    tokenAtPurge += tokenStorage.getToken()
                }
            val viewModel = authViewModel(apiClient, tokenStorage, purger)
            viewModel.authState.first { it is AuthState.Authenticated }

            val scope = tokenStorage.currentOfflineScope()!!
            store.clientIdentity(scope)
            store.upsertCachedDocument(scope, cachedDocumentRow())

            viewModel.logout()
            viewModel.authState.first { it is AuthState.Unauthenticated }
            purged.await()
            advanceUntilIdle()

            assertNull(tokenStorage.getUserId())
            assertEquals(listOf(scope), purgedScopes)
            assertEquals(listOf<String?>(null), tokenAtPurge)
            assertTrue(store.scopesWithState().none { it.first == scope })
        }

    @Test
    fun loginReachesAuthenticatedEvenWhenTheInactiveSweepThrows() =
        runTest {
            val tokenStorage = InMemoryTokenStorage()
            tokenStorage.saveServerUrl("https://example.com")

            val engine =
                MockEngine { request ->
                    when (request.url.encodedPath) {
                        "/api/v1/auth/login" ->
                            respond(authResponseJson(), HttpStatusCode.OK, jsonHeaders)
                        else -> respond("", HttpStatusCode.Unauthorized)
                    }
                }
            val apiClient = ApiClient(tokenStorage, engine = engine)
            val worker = testOutboxWorker()
            val viewModel =
                AuthViewModel(
                    ApiAuthRepository(apiClient.authApiService, apiClient.accountApiService),
                    tokenStorage,
                    testOfflineAccount(
                        tokenStorage,
                        worker = worker,
                        purger = ScopePurger(ThrowingScopesStore(SqlDelightOfflineStore(testOfflineDatabase()))),
                    ),
                    testSessionTransitions(tokenStorage, worker = worker),
                )
            viewModel.authState.first { it is AuthState.Unauthenticated }

            viewModel.updateLoginEmail("user@example.com")
            viewModel.updateLoginPassword("password123")
            viewModel.login()
            viewModel.authState.first { it is AuthState.Authenticated }

            assertEquals(false, worker.authPaused.value)
        }

    @Test
    fun loginPurgesInactiveScopesButKeepsTheActiveOne() =
        runTest {
            val tokenStorage = InMemoryTokenStorage()
            tokenStorage.saveServerUrl("https://example.com")

            val store = SqlDelightOfflineStore(testOfflineDatabase())
            val activeScope = "${normalizedOrigin("https://example.com")}|usr_01ABCDEF"
            val inactiveScope = "${normalizedOrigin("https://example.com")}|usr_OLD"
            store.clientIdentity(activeScope)
            store.upsertCachedDocument(activeScope, cachedDocumentRow())
            store.clientIdentity(inactiveScope)
            store.upsertCachedDocument(inactiveScope, cachedDocumentRow())

            val engine =
                MockEngine { request ->
                    when (request.url.encodedPath) {
                        "/api/v1/auth/login" ->
                            respond(
                                content = authResponseJson(),
                                status = HttpStatusCode.OK,
                                headers = jsonHeaders,
                            )
                        else -> respond("", HttpStatusCode.Unauthorized)
                    }
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val viewModel = authViewModel(apiClient, tokenStorage, ScopePurger(store))
            viewModel.authState.first { it is AuthState.Unauthenticated }

            viewModel.updateLoginEmail("user@example.com")
            viewModel.updateLoginPassword("password123")
            viewModel.login()
            viewModel.authState.first { it is AuthState.Authenticated }

            assertNotNull(store.cachedDocument(activeScope, "doc_1"))
            assertNull(store.cachedDocument(inactiveScope, "doc_1"))
            assertEquals(listOf(activeScope), store.scopesWithState().map { it.first })
        }

    private suspend fun signedInTokenStorage() =
        InMemoryTokenStorage().apply {
            saveServerUrl("https://example.com")
            saveToken("existing-token")
            saveRefreshToken("existing-refresh")
            saveExpiresAt(FAR_FUTURE_EXPIRY)
        }

    private fun sessionEngine() =
        MockEngine { request ->
            when (request.url.encodedPath) {
                "/api/v1/me" -> respond(profileResponseJson(), HttpStatusCode.OK, jsonHeaders)
                "/api/v1/auth/logout" -> respond("", HttpStatusCode.NoContent, jsonHeaders)
                else -> respond("", HttpStatusCode.OK)
            }
        }

    private fun authResponseJson() =
        """
        {
            "id": "usr_01ABCDEF",
            "object": "user",
            "email": "user@example.com",
            "display_name": "Test User",
            "email_verified": true,
            "onboarding_completed": true,
            "access_token": "test-token",
            "refresh_token": "test-refresh",
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

    private fun cachedDocumentRow() =
        CachedDocumentRow(
            "doc_1",
            "article",
            "Title",
            "{}",
            pinned = false,
            lastOpenedAt = 1L,
            lastSyncedAt = null,
            bytes = 5L,
        )

    private fun authViewModel(
        apiClient: ApiClient,
        tokenStorage: InMemoryTokenStorage,
        scopePurger: ScopePurger,
    ): AuthViewModel =
        AuthViewModel(
            ApiAuthRepository(apiClient.authApiService, apiClient.accountApiService),
            tokenStorage,
            testOfflineAccount(tokenStorage, purger = scopePurger),
            testSessionTransitions(tokenStorage),
        )

    /** Fails the first purge step that touches data, after purge_pending is already set. */
    private class ThrowingPurgeRowsStore(
        delegate: OfflineStore,
        private val attempted: CompletableDeferred<Unit>,
    ) : OfflineStore by delegate {
        override suspend fun purgeRows(scope: String) {
            attempted.complete(Unit)
            error("purge unavailable")
        }
    }

    /** Signals the last purge step so a test can wait for a purge that trails the sign-out. */
    private class PurgeFinishedStore(
        private val delegate: OfflineStore,
        private val finished: CompletableDeferred<Unit>,
    ) : OfflineStore by delegate {
        override suspend fun finishPurge(scope: String) {
            delegate.finishPurge(scope)
            finished.complete(Unit)
        }
    }

    private class ThrowingScopesStore(
        delegate: OfflineStore,
    ) : OfflineStore by delegate {
        override suspend fun scopesWithState(): List<Pair<String, Boolean>> = error("scope sweep unavailable")
    }

    companion object {
        private const val FAR_FUTURE_EXPIRY = 4_102_444_800L
    }
}
