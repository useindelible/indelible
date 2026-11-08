package app.indelible.auth.viewmodel

import app.indelible.core.offline.OutboxPayload
import app.indelible.core.offline.enqueue
import app.indelible.core.offline.enqueueNote
import app.indelible.core.offline.enqueueProgress
import app.indelible.core.offline.signedInStore
import app.indelible.core.storage.InMemoryTokenStorage
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals

private const val SERVER = "http://a.test"
private const val OTHER_SERVER = "http://b.test"

class OfflineAccountWaitingTest {
    private suspend fun storageFor(server: String) = InMemoryTokenStorage().apply { saveServerUrl(server) }

    @Test
    fun countsLiveRowsOfTheServersScopesOnly() =
        runTest {
            val s = signedInStore(scope = "$SERVER|u1")
            s.enqueue(OutboxPayload.DocumentNote("one", null))
            s.store.enqueueNote(s.switchTo("$SERVER|u2"), "doc_2")
            s.store.enqueueNote(s.switchTo("$OTHER_SERVER|u1"), "doc_3")
            val account = testOfflineAccount(storageFor("$SERVER/"), store = s.store)

            assertEquals(2, account.waitingChanges().first())
            assertEquals(1, testOfflineAccount(storageFor(OTHER_SERVER), store = s.store).waitingChanges().first())
        }

    @Test
    fun purgePendingScopesAreNotCounted() =
        runTest {
            val s = signedInStore(scope = "$SERVER|u1")
            s.enqueue(OutboxPayload.DocumentNote("one", null))
            s.store.enqueueNote(s.switchTo("$SERVER|u2"), "doc_2")
            s.store.setPurgePending("$SERVER|u1", true)

            assertEquals(1, testOfflineAccount(storageFor(SERVER), store = s.store).waitingChanges().first())
        }

    @Test
    fun supersededRowsAreNotCounted() =
        runTest {
            val s = signedInStore(scope = "$SERVER|u1")
            s.enqueue(OutboxPayload.DocumentNote("older", null))
            s.store.remove(s.scope, s.enqueue(OutboxPayload.DocumentNote("newer", null)))
            s.enqueueProgress(basisPoints = 1_000)

            assertEquals(1, testOfflineAccount(storageFor(SERVER), store = s.store).waitingChanges().first())
        }

    @Test
    fun noStoredServerMeansNothingWaiting() =
        runTest {
            val s = signedInStore(scope = "$SERVER|u1")
            s.enqueue(OutboxPayload.DocumentNote("one", null))

            assertEquals(0, testOfflineAccount(InMemoryTokenStorage(), store = s.store).waitingChanges().first())
        }
}
