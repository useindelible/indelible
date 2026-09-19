package app.indelible.auth.viewmodel

import app.indelible.auth.repository.ApiAuthRepository
import app.indelible.core.model.AuthUser
import app.indelible.core.network.ApiClient
import app.indelible.core.offline.ScopePurger
import app.indelible.core.offline.SessionRegistry
import app.indelible.core.offline.SessionTransitions
import app.indelible.core.offline.SqlDelightOfflineStore
import app.indelible.core.offline.currentOfflineScope
import app.indelible.core.offline.testOutboxWorker
import app.indelible.core.storage.InMemoryTokenStorage
import app.indelible.db.testOfflineDatabase
import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.MockRequestHandleScope
import io.ktor.client.engine.mock.respond
import io.ktor.client.request.HttpResponseData
import io.ktor.http.HttpHeaders
import io.ktor.http.HttpStatusCode
import io.ktor.http.headersOf
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.awaitCancellation
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import kotlinx.coroutines.withContext
import kotlinx.serialization.json.Json
import okio.IOException
import kotlin.test.AfterTest
import kotlin.test.BeforeTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlin.test.assertNull

private const val SERVER = "https://example.com"
private const val FAR_FUTURE_EXPIRY = 4_102_444_800L
private const val SETTLE_MS = 200L
private val jsonHeaders = headersOf(HttpHeaders.ContentType, "application/json")

private fun profileJson(
    name: String,
    id: String = "usr_A",
) = """{"id":"$id","object":"user","email":"a@example.com","display_name":"$name","email_verified":true,""" +
    """"onboarding_completed":true,"has_password":true,"locale":"en","theme":"auto","timezone":"UTC",""" +
    """"created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z"}"""

private val PROFILE_B = profileJson("User B", id = "usr_B")

private val AUTH_B =
    """{"id":"usr_B","object":"user","email":"b@example.com","display_name":"User B","email_verified":true,""" +
        """"onboarding_completed":true,"access_token":"token-B","refresh_token":"refresh-B",""" +
        """"expires_at":$FAR_FUTURE_EXPIRY}"""

/** How the server answers user A's profile request, and how many profile requests of anyone's it saw. */
private class Answers {
    var profileRequests = 0
    var profileForA: suspend MockRequestHandleScope.() -> HttpResponseData = {
        respond(profileJson("Fresh A"), HttpStatusCode.OK, jsonHeaders)
    }
}

private class Start(
    val tokenStorage: InMemoryTokenStorage,
    val sessions: SessionTransitions,
    val offline: OfflineAccount,
    val online: MutableStateFlow<Boolean>,
    val answers: Answers,
    private val apiClient: ApiClient,
) {
    var viewModel: AuthViewModel? = null

    fun open(): AuthViewModel {
        val repository = ApiAuthRepository(apiClient.authApiService, apiClient.accountApiService)
        return AuthViewModel(repository, tokenStorage, offline, sessions).also { viewModel = it }
    }
}

@OptIn(ExperimentalCoroutinesApi::class)
class AuthOfflineStartTest {
    @BeforeTest
    fun setUp() {
        Dispatchers.setMain(UnconfinedTestDispatcher())
    }

    @AfterTest
    fun tearDown() {
        Dispatchers.resetMain()
    }

    /** User A signed in on an earlier run, with [kept] as the profile that run kept, if any. */
    private suspend fun start(
        signedIn: Boolean = true,
        kept: String? = "Cached A",
    ): Start {
        val registry = SessionRegistry()
        val tokenStorage = InMemoryTokenStorage().apply { saveServerUrl(SERVER) }
        val store = SqlDelightOfflineStore(testOfflineDatabase(), registry = registry)
        val worker = testOutboxWorker()
        val sessions = SessionTransitions(registry, tokenStorage, store, worker)
        val online = MutableStateFlow(true)
        val answers = Answers()
        lateinit var start: Start
        val engine =
            MockEngine { request ->
                val path = request.url.encodedPath
                val forA = request.headers[HttpHeaders.Authorization] == "Bearer token-A"
                if (path == "/api/v1/me") answers.profileRequests++
                when {
                    path == "/api/v1/me" && forA -> answers.profileForA(this)
                    path == "/api/v1/auth/login" -> respond(AUTH_B, HttpStatusCode.OK, jsonHeaders)
                    path == "/api/v1/me" -> respond(PROFILE_B, HttpStatusCode.OK, jsonHeaders)
                    else -> respond("", HttpStatusCode.Unauthorized)
                }
            }
        val apiClient = ApiClient(tokenStorage, { epoch -> start.viewModel?.forceLogout(epoch) }, engine, registry)
        val offline = OfflineAccount(store, tokenStorage, worker, ScopePurger(store), online)
        start = Start(tokenStorage, sessions, offline, online, answers, apiClient)
        if (signedIn) {
            tokenStorage.saveToken("token-A")
            tokenStorage.saveRefreshToken("refresh-A")
            tokenStorage.saveExpiresAt(FAR_FUTURE_EXPIRY)
            tokenStorage.saveUserId("usr_A")
            val scope = checkNotNull(tokenStorage.currentOfflineScope())
            store.clientIdentity(scope)
            kept?.let { store.keepProfile(scope, Json.encodeToString(AuthUser.serializer(), userA(it))) }
            sessions.restore()
        }
        return start
    }

    private fun userA(name: String) =
        AuthUser(
            id = "usr_A",
            email = "a@example.com",
            displayName = name,
            emailVerified = true,
            onboardingCompleted = true,
        )

    private suspend fun AuthViewModel.settledUser(): String =
        assertIs<AuthState.Authenticated>(authState.first { it !is AuthState.Loading }).user.displayName

    private suspend fun settle() = withContext(Dispatchers.Default) { delay(SETTLE_MS) }

    private fun AuthViewModel.logInAsB() {
        updateLoginEmail("b@example.com")
        updateLoginPassword("password123")
        login()
    }

    @Test
    fun cached_profile_enters_immediately_while_session_request_never_responds() =
        runTest {
            val s = start()
            s.answers.profileForA = { awaitCancellation() }

            assertEquals("Cached A", s.open().settledUser())
        }

    @Test
    fun revalidation_success_publishes_fresh_user_without_loading() =
        runTest {
            val s = start()
            val release = CompletableDeferred<Unit>()
            s.answers.profileForA = {
                release.await()
                respond(profileJson("Fresh A"), HttpStatusCode.OK, jsonHeaders)
            }
            val viewModel = s.open()
            assertEquals("Cached A", viewModel.settledUser())

            release.complete(Unit)

            val fresh = viewModel.authState.first { (it as? AuthState.Authenticated)?.user?.displayName == "Fresh A" }
            assertIs<AuthState.Authenticated>(fresh)
            assertEquals(0L, s.sessions.epoch())
        }

    @Test
    fun confirmed_auth_rejection_signs_out() =
        runTest {
            val s = start()
            val release = CompletableDeferred<Unit>()
            s.answers.profileForA = {
                release.await()
                respond("", HttpStatusCode.Unauthorized)
            }
            val viewModel = s.open()
            assertEquals("Cached A", viewModel.settledUser())

            release.complete(Unit)

            viewModel.authState.first { it is AuthState.Unauthenticated }
            assertNull(s.tokenStorage.getToken())
        }

    @Test
    fun revalidation_after_newer_sign_in_is_ignored() =
        runTest {
            val s = start()
            val release = CompletableDeferred<Unit>()
            val answered = CompletableDeferred<Unit>()
            s.answers.profileForA = {
                release.await()
                answered.complete(Unit)
                respond(profileJson("Fresh A"), HttpStatusCode.OK, jsonHeaders)
            }
            val viewModel = s.open()
            assertEquals("Cached A", viewModel.settledUser())
            viewModel.logInAsB()
            viewModel.authState.first { (it as? AuthState.Authenticated)?.user?.id == "usr_B" }

            release.complete(Unit)
            answered.await()
            settle()

            assertEquals("usr_B", assertIs<AuthState.Authenticated>(viewModel.authState.value).user.id)
        }

    @Test
    fun no_cached_profile_keeps_todays_path() =
        runTest {
            val s = start(kept = null)
            val release = CompletableDeferred<Unit>()
            s.answers.profileForA = {
                release.await()
                respond(profileJson("Fresh A"), HttpStatusCode.OK, jsonHeaders)
            }
            val viewModel = s.open()
            settle()
            assertIs<AuthState.Loading>(viewModel.authState.value)

            release.complete(Unit)

            assertEquals("Fresh A", viewModel.settledUser())
            assertEquals(1L, s.sessions.epoch())
        }

    @Test
    fun revalidation_retries_when_connectivity_returns() =
        runTest {
            val s = start()
            s.answers.profileForA = {
                if (s.answers.profileRequests == 1) throw IOException("offline")
                respond(profileJson("Fresh A"), HttpStatusCode.OK, jsonHeaders)
            }
            val viewModel = s.open()
            assertEquals("Cached A", viewModel.settledUser())
            settle()
            assertEquals(1, s.answers.profileRequests)

            s.online.value = false
            settle()
            s.online.value = true

            viewModel.authState.first { (it as? AuthState.Authenticated)?.user?.displayName == "Fresh A" }
            assertEquals(2, s.answers.profileRequests)
        }

    @Test
    fun revalidation_stops_once_another_account_signs_in() =
        runTest {
            val s = start()
            s.answers.profileForA = { throw IOException("offline") }
            val viewModel = s.open()
            assertEquals("Cached A", viewModel.settledUser())
            settle()
            viewModel.logInAsB()
            viewModel.authState.first { (it as? AuthState.Authenticated)?.user?.id == "usr_B" }

            s.online.value = false
            settle()
            s.online.value = true
            settle()

            assertEquals(1, s.answers.profileRequests)
        }

    @Test
    fun published_profile_is_kept_for_the_next_start() =
        runTest {
            val s = start(signedIn = false)
            val viewModel = s.open()
            viewModel.authState.first { it is AuthState.Unauthenticated }

            viewModel.logInAsB()
            viewModel.authState.first { it is AuthState.Authenticated }
            settle()

            assertEquals("usr_B", s.offline.cachedUser()?.id)
        }
}
