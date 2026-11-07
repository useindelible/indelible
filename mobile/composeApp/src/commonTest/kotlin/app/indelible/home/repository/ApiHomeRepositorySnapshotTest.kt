package app.indelible.home.repository

import app.indelible.core.network.HomeApiService
import app.indelible.core.offline.DownloadHarness
import app.indelible.core.offline.ListSnapshots
import app.indelible.core.offline.SessionState
import app.indelible.core.offline.downloadHarness
import app.indelible.core.offline.json
import io.ktor.client.engine.mock.respond
import io.ktor.http.HttpStatusCode
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.async
import kotlinx.coroutines.test.runCurrent
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

private const val HOME = "/api/v1/home"

class ApiHomeRepositorySnapshotTest {
    private fun DownloadHarness.repository(): ApiHomeRepository {
        val current = { signedIn.registry.current.value.session }
        return ApiHomeRepository(HomeApiService(transport), ListSnapshots(store, current) { 1L })
    }

    private suspend fun DownloadHarness.kept(key: String): List<String> =
        store.list(session.scope, key).map { it.documentId }

    @Test
    fun dashboardKeepsContinueAndRecent() =
        runTest {
            val harness = downloadHarness()
            harness.server.route(HOME) { json(dashboardJson(listOf("doc_b", "doc_a"), listOf("doc_c"))) }

            harness.repository().getDashboard()

            assertEquals(listOf("doc_b", "doc_a"), harness.kept("home:continue"))
            assertEquals(listOf("doc_c"), harness.kept("home:recent"))
        }

    @Test
    fun anAbsentWidgetClearsItsKey() =
        runTest {
            val harness = downloadHarness()
            val repository = harness.repository()
            harness.server.route(HOME) { json(dashboardJson(listOf("doc_a"), listOf("doc_c"))) }
            repository.getDashboard()
            harness.server.route(HOME) { json(dashboardJson(null, listOf("doc_d"))) }

            repository.getDashboard()

            assertTrue(harness.kept("home:continue").isEmpty())
            assertEquals(listOf("doc_d"), harness.kept("home:recent"))
        }

    @Test
    fun aFailedDashboardKeepsNothing() =
        runTest {
            val harness = downloadHarness()
            val repository = harness.repository()
            harness.server.route(HOME) { json(dashboardJson(listOf("doc_a"), listOf("doc_c"))) }
            repository.getDashboard()
            harness.server.route(HOME) { respond("", HttpStatusCode.ServiceUnavailable) }

            assertTrue(repository.getDashboard().isFailure)
            assertEquals(listOf("doc_a"), harness.kept("home:continue"))
        }

    @Test
    fun signedOutKeepsNothing() =
        runTest {
            val harness = downloadHarness()
            harness.server.route(HOME) { json(dashboardJson(listOf("doc_a"), null)) }
            val repository = harness.repository()
            harness.signedIn.registry.publish(SessionState(harness.session.epoch + 1, null))

            repository.getDashboard()

            assertTrue(harness.kept("home:continue").isEmpty())
        }

    @Test
    fun aResponseThatLandsAfterALoginToAnotherAccountIsNotKept() =
        runTest {
            val harness = downloadHarness()
            val gate = CompletableDeferred<Unit>()
            harness.server.route(HOME) {
                gate.await()
                json(dashboardJson(listOf("doc_a"), listOf("doc_c")))
            }
            val pending = async { harness.repository().getDashboard() }
            runCurrent()

            val other = harness.signedIn.switchTo("http://a.test|u2")
            gate.complete(Unit)
            pending.await()

            assertTrue(harness.kept("home:continue").isEmpty())
            assertTrue(harness.store.list(other.scope, "home:continue").isEmpty())
        }
}
