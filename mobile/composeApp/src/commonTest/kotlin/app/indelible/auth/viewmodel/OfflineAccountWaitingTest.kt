package app.indelible.auth.viewmodel

import app.indelible.core.offline.OutboxPayload
import app.indelible.core.offline.SignedIn
import app.indelible.core.offline.enqueue
import app.indelible.core.offline.enqueueNote
import app.indelible.core.offline.enqueueProgress
import app.indelible.core.offline.rejected
import app.indelible.core.offline.signedInStore
import app.indelible.core.storage.InMemoryTokenStorage
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals

private const val SERVER = "http://a.test"
private const val OTHER_SERVER = "http://b.test"

class OfflineAccountWaitingTest {
    private suspend fun SignedIn.waiting(server: String?): Int =
        testOfflineAccount(InMemoryTokenStorage(), store = store).waitingChanges(flowOf(server)).first()

    @Test
    fun countsLiveRowsOfTheServersScopesOnly() =
        runTest {
            val s = signedInStore(scope = "$SERVER|u1")
            s.enqueue(OutboxPayload.DocumentNote("one", null))
            s.store.enqueueNote(s.switchTo("$SERVER|u2"), "doc_2")
            s.store.enqueueNote(s.switchTo("$OTHER_SERVER|u1"), "doc_3")

            assertEquals(2, s.waiting("$SERVER/"))
            assertEquals(1, s.waiting(OTHER_SERVER))
        }

    @Test
    fun purgePendingScopesAreNotCounted() =
        runTest {
            val s = signedInStore(scope = "$SERVER|u1")
            s.enqueue(OutboxPayload.DocumentNote("one", null))
            s.store.enqueueNote(s.switchTo("$SERVER|u2"), "doc_2")
            s.store.setPurgePending("$SERVER|u1", true)

            assertEquals(1, s.waiting(SERVER))
        }

    @Test
    fun supersededRowsAreNotCounted() =
        runTest {
            val s = signedInStore(scope = "$SERVER|u1")
            s.enqueue(OutboxPayload.DocumentNote("older", null))
            s.store.remove(s.scope, s.enqueue(OutboxPayload.DocumentNote("newer", null)))
            s.enqueueProgress(basisPoints = 1_000)

            assertEquals(1, s.waiting(SERVER))
        }

    @Test
    fun noStoredServerMeansNothingWaiting() =
        runTest {
            val s = signedInStore(scope = "$SERVER|u1")
            s.enqueue(OutboxPayload.DocumentNote("one", null))

            assertEquals(0, s.waiting(null))
        }

    @Test
    fun readingEventsCountAsOneChangePerDocumentAndFailedRowsNotAtAll() =
        runTest {
            val s = signedInStore(scope = "$SERVER|u1")
            s.enqueueProgress(basisPoints = 1_000)
            s.enqueueProgress(basisPoints = 2_000)
            s.store.markFailed(s.scope, s.enqueue(OutboxPayload.DocumentNote("no", null)), rejected("no"))

            assertEquals(1, s.waiting(SERVER))
        }

    @Test
    fun theCountFollowsTheServerChosenOnTheLoginScreen() =
        runTest {
            val s = signedInStore(scope = "$SERVER|u1")
            s.enqueue(OutboxPayload.DocumentNote("one", null))
            val server = MutableStateFlow<String?>(OTHER_SERVER)
            val account = testOfflineAccount(InMemoryTokenStorage(), store = s.store)

            assertEquals(0, account.waitingChanges(server).first())
            server.value = SERVER
            assertEquals(1, account.waitingChanges(server).first())
        }
}
