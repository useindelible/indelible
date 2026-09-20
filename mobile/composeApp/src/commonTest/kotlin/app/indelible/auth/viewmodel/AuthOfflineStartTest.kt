package app.indelible.auth.viewmodel

import io.ktor.client.engine.mock.respond
import io.ktor.http.HttpStatusCode
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.awaitCancellation
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import okio.IOException
import kotlin.test.AfterTest
import kotlin.test.BeforeTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlin.test.assertNull

@OptIn(ExperimentalCoroutinesApi::class)
class AuthOfflineStartTest {
    @BeforeTest
    fun setUp() {
        // settle() resumes onto Main from a real thread, which an unconfined main dispatcher refuses.
        Dispatchers.setMain(StandardTestDispatcher())
    }

    @AfterTest
    fun tearDown() {
        Dispatchers.resetMain()
    }

    @Test
    fun cached_profile_enters_immediately_while_session_request_never_responds() =
        runTest {
            val s = startOffline()
            s.answers.profileForA = { awaitCancellation() }

            assertEquals("Cached A", s.open().settledUser())
        }

    @Test
    fun revalidation_success_publishes_fresh_user_without_loading() =
        runTest {
            val s = startOffline()
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
            val s = startOffline()
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
            val s = startOffline()
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
            val s = startOffline(kept = null)
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
            val s = startOffline()
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
            val s = startOffline()
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
            val s = startOffline(signedIn = false)
            val viewModel = s.open()
            viewModel.authState.first { it is AuthState.Unauthenticated }

            viewModel.logInAsB()
            viewModel.authState.first { it is AuthState.Authenticated }
            settle()

            assertEquals("usr_B", s.offline.cachedUser()?.id)
        }
}
