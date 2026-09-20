package app.indelible.auth.viewmodel

import app.indelible.auth.repository.ApiAuthRepository
import app.indelible.core.model.AuthUser
import app.indelible.core.network.ApiClient
import app.indelible.core.offline.OfflineStore
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
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.withContext
import kotlinx.serialization.json.Json
import kotlin.test.assertIs

private const val SERVER = "https://example.com"
private const val FAR_FUTURE_EXPIRY = 4_102_444_800L
private const val SETTLE_MS = 200L
internal val jsonHeaders = headersOf(HttpHeaders.ContentType, "application/json")

internal fun profileJson(
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
internal class Answers {
    var profileRequests = 0
    var profileForA: suspend MockRequestHandleScope.() -> HttpResponseData = {
        respond(profileJson("Fresh A"), HttpStatusCode.OK, jsonHeaders)
    }
}

internal class Start(
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

/** User A signed in on an earlier run, with [kept] as the profile that run kept, if any. */
internal suspend fun startOffline(
    signedIn: Boolean = true,
    kept: String? = "Cached A",
    profileRead: CompletableDeferred<Unit>? = null,
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
    val profiles =
        object : OfflineStore by store {
            override suspend fun profile(scope: String): String? = store.profile(scope).also { profileRead?.await() }
        }
    val offline = OfflineAccount(profiles, tokenStorage, worker, ScopePurger(store), online)
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

internal suspend fun AuthViewModel.settledUser(): String =
    assertIs<AuthState.Authenticated>(authState.first { it !is AuthState.Loading }).user.displayName

internal suspend fun settle() = withContext(Dispatchers.Default) { delay(SETTLE_MS) }

internal fun AuthViewModel.logInAsB() {
    updateLoginEmail("b@example.com")
    updateLoginPassword("password123")
    login()
}
