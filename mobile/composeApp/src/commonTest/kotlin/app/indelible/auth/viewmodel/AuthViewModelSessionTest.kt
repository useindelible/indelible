package app.indelible.auth.viewmodel

import app.indelible.auth.repository.ApiAuthRepository
import app.indelible.core.network.ApiClient
import app.indelible.core.offline.ScopePurger
import app.indelible.core.offline.SessionRegistry
import app.indelible.core.offline.SessionTransitions
import app.indelible.core.offline.SqlDelightOfflineStore
import app.indelible.core.offline.StaleWriteException
import app.indelible.core.offline.enqueueNote
import app.indelible.core.offline.testOutboxWorker
import app.indelible.core.storage.InMemoryTokenStorage
import app.indelible.db.testOfflineDatabase
import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.respond
import io.ktor.http.HttpHeaders
import io.ktor.http.HttpMethod
import io.ktor.http.HttpStatusCode
import io.ktor.http.content.TextContent
import io.ktor.http.headersOf
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.launch
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import kotlinx.coroutines.withContext
import kotlin.test.AfterTest
import kotlin.test.BeforeTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertIs
import kotlin.test.assertNull
import kotlin.test.assertSame
import kotlin.test.assertTrue

private const val FAR_FUTURE_EXPIRY = 4_102_444_800L
private const val SERVER = "https://example.com"
private const val FAILURE_SETTLE_MS = 200L

@OptIn(ExperimentalCoroutinesApi::class)
class AuthViewModelSessionTest {
    private val jsonHeaders = headersOf(HttpHeaders.ContentType, "application/json")

    @BeforeTest
    fun setUp() {
        Dispatchers.setMain(UnconfinedTestDispatcher())
    }

    @AfterTest
    fun tearDown() {
        Dispatchers.resetMain()
    }

    /** Requests for user A wait on [gateA], whose value is the status A's profile fetch gets; B answers at once. */
    private class Gates(
        val a: CompletableDeferred<HttpStatusCode>,
        val profile: CompletableDeferred<Unit>,
    )

    private class Fixture(
        val registry: SessionRegistry,
        val tokenStorage: InMemoryTokenStorage,
        val store: SqlDelightOfflineStore,
        val sessions: SessionTransitions,
        val viewModel: AuthViewModel,
        val gates: Gates,
    ) {
        val gateA get() = gates.a
        val profileGate get() = gates.profile
    }

    private suspend fun fixture(signedInAs: String? = null): Fixture {
        val registry = SessionRegistry()
        val tokenStorage = InMemoryTokenStorage()
        tokenStorage.saveServerUrl(SERVER)
        if (signedInAs != null) {
            tokenStorage.saveToken("token-$signedInAs")
            tokenStorage.saveRefreshToken("refresh-$signedInAs")
            tokenStorage.saveExpiresAt(FAR_FUTURE_EXPIRY)
            tokenStorage.saveUserId("usr_$signedInAs")
        }
        val store = SqlDelightOfflineStore(testOfflineDatabase(), registry = registry)
        val gateA = CompletableDeferred<HttpStatusCode>()
        val profileGate = CompletableDeferred<Unit>()
        val engine =
            MockEngine { request ->
                val body = (request.body as? TextContent)?.text.orEmpty()
                val bearer = request.headers[HttpHeaders.Authorization]
                val forA = body.contains("a@example.com") || bearer == "Bearer token-A"
                val aProfileStatus = if (forA) gateA.await() else HttpStatusCode.OK
                if (request.url.encodedPath == "/api/v1/me" && request.method != HttpMethod.Get) profileGate.await()
                val user = if (forA) "A" else "B"
                when (request.url.encodedPath) {
                    "/api/v1/auth/login" -> respond(authResponseJson(user), HttpStatusCode.OK, jsonHeaders)
                    "/api/v1/auth/logout" -> respond("", HttpStatusCode.NoContent)
                    "/api/v1/me" -> respond(profileResponseJson(user), aProfileStatus, jsonHeaders)
                    else -> respond("", HttpStatusCode.OK)
                }
            }
        val apiClient = ApiClient(tokenStorage, engine = engine, registry = registry)
        val worker = testOutboxWorker()
        val sessions = SessionTransitions(registry, tokenStorage, store, worker)
        val viewModel =
            AuthViewModel(
                ApiAuthRepository(apiClient.authApiService, apiClient.accountApiService),
                tokenStorage,
                worker,
                ScopePurger(store),
                sessions,
            )
        return Fixture(registry, tokenStorage, store, sessions, viewModel, Gates(gateA, profileGate))
    }

    private fun AuthViewModel.loginAs(email: String) {
        updateLoginEmail(email)
        updateLoginPassword("password123")
        login()
    }

    private suspend fun Fixture.assertSignedInAsB() {
        val b = checkNotNull(registry.current.value.session)
        assertEquals("usr_B", tokenStorage.getUserId())
        assertEquals("token-B", tokenStorage.getToken())
        val state = assertIs<AuthState.Authenticated>(viewModel.authState.first { it is AuthState.Authenticated })
        assertEquals("usr_B", state.user.id)
        store.enqueueNote(b, "doc_1")
        assertEquals(1, store.pendingOrdered(b.scope).size)
        assertSame(b, registry.current.value.session)
    }

    @Test
    fun delayedLoginResultForAAfterBSignedInIsRejectedAndBStaysSignedIn() =
        runTest {
            val f = fixture()
            f.viewModel.authState.first { it is AuthState.Unauthenticated }

            f.viewModel.loginAs("a@example.com")
            f.viewModel.loginAs("b@example.com")
            f.viewModel.authState.first { it is AuthState.Authenticated }
            val epochAfterB = f.registry.current.value.epoch

            f.gateA.complete(HttpStatusCode.OK)

            assertEquals(epochAfterB, f.registry.current.value.epoch)
            f.assertSignedInAsB()
        }

    @Test
    fun delayedSessionRefetchForAAfterBSignedInIsRejected() =
        runTest {
            val f = fixture(signedInAs = "A")
            assertIs<AuthState.Loading>(f.viewModel.authState.value)

            f.viewModel.loginAs("b@example.com")
            f.viewModel.authState.first { it is AuthState.Authenticated }

            f.gateA.complete(HttpStatusCode.OK)

            f.assertSignedInAsB()
        }

    @Test
    fun delayedProfileResultForAAfterLogoutAndBSignInIsRejected() =
        runTest {
            val f = fixture(signedInAs = "A")
            f.gateA.complete(HttpStatusCode.OK)
            f.viewModel.authState.first { it is AuthState.Authenticated }
            val a = checkNotNull(f.registry.current.value.session)
            val completed = CompletableDeferred<Boolean>()
            f.viewModel.updateProfile("Renamed A") { completed.complete(it) }

            f.viewModel.logout()
            f.viewModel.authState.first { it is AuthState.Unauthenticated }
            f.viewModel.loginAs("b@example.com")
            f.viewModel.authState.first { it is AuthState.Authenticated }
            f.profileGate.complete(Unit)

            assertEquals(false, completed.await())
            assertFailsWith<StaleWriteException> { f.store.enqueueNote(a, "doc_1") }
            f.assertSignedInAsB()
        }

    @Test
    fun forcedLogoutForAThatRunsAfterBSignedInLeavesBSignedIn() =
        runTest {
            val f = fixture(signedInAs = "A")
            f.gateA.complete(HttpStatusCode.OK)
            f.viewModel.authState.first { it is AuthState.Authenticated }
            val epochA = f.registry.current.value.epoch

            f.viewModel.logout()
            f.viewModel.authState.first { it is AuthState.Unauthenticated }
            f.viewModel.loginAs("b@example.com")
            f.viewModel.authState.first { it is AuthState.Authenticated }
            val b = checkNotNull(f.registry.current.value.session)

            f.viewModel.forceLogout(epochA)

            assertSame(b, f.registry.current.value.session)
            f.assertSignedInAsB()
        }

    @Test
    fun startupFailureForAAfterBSignedInDoesNotSignBOut() =
        runTest {
            val f = fixture(signedInAs = "A")
            f.viewModel.initialize()
            f.viewModel.loginAs("b@example.com")
            f.viewModel.authState.first { it is AuthState.Authenticated }

            f.gateA.complete(HttpStatusCode.InternalServerError)
            // The mock engine answers A off the test dispatcher, so the failure lands a little later.
            withContext(Dispatchers.Default) { delay(FAILURE_SETTLE_MS) }

            val state = assertIs<AuthState.Authenticated>(f.viewModel.authState.value)
            assertEquals("usr_B", state.user.id)
            f.assertSignedInAsB()
        }

    @Test
    fun logoutStartedWhileAnotherTransitionIsOpenStillPurgesTheOutgoingScope() =
        runTest {
            val f = fixture(signedInAs = "A")
            f.gateA.complete(HttpStatusCode.OK)
            f.viewModel.authState.first { it is AuthState.Authenticated }
            val a = checkNotNull(f.registry.current.value.session)
            f.store.enqueueNote(a, "doc_1")
            val opened = CompletableDeferred<Unit>()
            val hold = CompletableDeferred<Unit>()
            val relogin =
                launch {
                    f.sessions.transition(a.epoch) {
                        opened.complete(Unit)
                        hold.await()
                        f.tokenStorage.saveUserId("usr_A")
                    }
                }
            opened.await()

            f.viewModel.logout()
            hold.complete(Unit)
            relogin.join()
            f.viewModel.authState.first { it is AuthState.Unauthenticated }

            assertTrue(f.store.scopesWithState().isEmpty())
            assertNull(f.tokenStorage.getToken())
        }

    @Test
    fun fullLoginChainOpensExactlyOneTransition() =
        runTest {
            val f = fixture()
            f.viewModel.authState.first { it is AuthState.Unauthenticated }
            val before = f.registry.current.value.epoch

            f.viewModel.loginAs("b@example.com")
            f.viewModel.authState.first { it is AuthState.Authenticated }

            assertEquals(before + 1, f.registry.current.value.epoch)
        }

    @Test
    fun logoutPurgesInsideTheTransitionAndAWriterHoldingTheOldSessionFails() =
        runTest {
            val f = fixture(signedInAs = "A")
            f.gateA.complete(HttpStatusCode.OK)
            f.viewModel.authState.first { it is AuthState.Authenticated }
            val a = checkNotNull(f.registry.current.value.session)
            f.store.enqueueNote(a, "doc_1")

            f.viewModel.logout()
            f.viewModel.authState.first { it is AuthState.Unauthenticated }

            assertNull(f.registry.current.value.session)
            assertEquals(a.epoch + 1, f.registry.current.value.epoch)
            assertFailsWith<StaleWriteException> { f.store.enqueueNote(a, "doc_2") }
            assertTrue(f.store.scopesWithState().isEmpty())
            assertNull(f.tokenStorage.getToken())
        }

    @Test
    fun invalidatedSessionIsClearedByAForcedLogoutForThatEpoch() =
        runTest {
            val f = fixture(signedInAs = "A")
            f.gateA.complete(HttpStatusCode.OK)
            f.viewModel.authState.first { it is AuthState.Authenticated }
            val epochA = f.registry.current.value.epoch

            assertTrue(f.registry.invalidate(epochA))
            f.viewModel.forceLogout(epochA)
            f.viewModel.authState.first { it is AuthState.Unauthenticated }

            assertNull(f.tokenStorage.getToken())
            assertEquals(epochA + 1, f.registry.current.value.epoch)
        }

    private fun authResponseJson(user: String) =
        """
        {
            "id": "usr_$user",
            "object": "user",
            "email": "${user.lowercase()}@example.com",
            "display_name": "User $user",
            "email_verified": true,
            "onboarding_completed": true,
            "access_token": "token-$user",
            "refresh_token": "refresh-$user",
            "expires_at": $FAR_FUTURE_EXPIRY
        }
        """.trimIndent()

    private fun profileResponseJson(user: String) =
        """
        {
            "id": "usr_$user",
            "object": "user",
            "email": "${user.lowercase()}@example.com",
            "display_name": "User $user",
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
}
