package app.indelible.auth.viewmodel

import io.ktor.client.engine.mock.respond
import io.ktor.http.HttpStatusCode
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.launch
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import okio.IOException
import kotlin.test.AfterTest
import kotlin.test.BeforeTest
import kotlin.test.Test
import kotlin.test.assertEquals

@OptIn(ExperimentalCoroutinesApi::class)
class AuthOfflineRaceTest {
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
    fun a_logout_during_the_profile_read_is_not_undone() =
        runTest {
            val read = CompletableDeferred<Unit>()
            val viewModel = startOffline(profileRead = read).open()
            settle()
            viewModel.forceLogout()
            viewModel.authState.first { it is AuthState.Unauthenticated }
            val after = mutableListOf<AuthState>()
            backgroundScope.launch(UnconfinedTestDispatcher(testScheduler)) {
                viewModel.authState.collect { after += it }
            }

            read.complete(Unit)
            settle()

            assertEquals(listOf<AuthState>(AuthState.Unauthenticated), after)
        }

    @Test
    fun revalidation_catches_a_reconnection_during_the_failing_request() =
        runTest {
            val s = startOffline()
            val inFlight = CompletableDeferred<Unit>()
            val fail = CompletableDeferred<Unit>()
            s.answers.profileForA = {
                if (s.answers.profileRequests == 1) {
                    inFlight.complete(Unit)
                    fail.await()
                    throw IOException("offline")
                }
                respond(profileJson("Fresh A"), HttpStatusCode.OK, jsonHeaders)
            }
            s.open()
            inFlight.await()
            s.online.value = false
            settle()
            s.online.value = true
            settle()
            fail.complete(Unit)
            settle()

            assertEquals(2, s.answers.profileRequests)
        }
}
