package app.indelible.core.offline

import app.indelible.db.OfflineDatabase
import app.indelible.db.testOfflineDatabase
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.Deferred
import kotlinx.coroutines.job
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.TestScope
import kotlinx.coroutines.test.runCurrent

/**
 * A sender whose next send can be held open until the test releases it. An unqueued send
 * succeeds, so a test that forgets an outcome never spins a recovery timer under virtual time.
 */
class GatedSender : OutboxSender {
    val calls = mutableListOf<List<OutboxRow>>()
    private val outcomes = ArrayDeque<SendOutcome>()
    private var hold: Hold? = null

    private class Hold {
        val started = CompletableDeferred<Unit>()
        val release = CompletableDeferred<Unit>()
    }

    fun enqueueOutcome(outcome: SendOutcome) {
        outcomes.addLast(outcome)
    }

    /** Returns a deferred that completes once the held send has recorded its batch. */
    fun holdNextSend(): Deferred<Unit> = Hold().also { hold = it }.started

    fun release() {
        hold?.release?.complete(Unit)
    }

    override suspend fun send(
        session: Session,
        clientId: String,
        batch: List<OutboxRow>,
    ): SendOutcome {
        calls += batch
        hold?.let { current ->
            current.started.complete(Unit)
            current.release.await()
            if (hold === current) hold = null
        }
        return outcomes.removeFirstOrNull() ?: SendOutcome.Success
    }
}

/** A store whose every hop runs on the test scheduler, so a whole pass completes inside runCurrent(). */
fun TestScope.testStore(database: OfflineDatabase = testOfflineDatabase()): SqlDelightOfflineStore =
    SqlDelightOfflineStore(database, dispatcher = StandardTestDispatcher(testScheduler))

fun TestScope.testWorker(
    store: OfflineStore,
    sender: OutboxSender,
    registry: SessionRegistry,
    clock: () -> Long = { testScheduler.currentTime },
): OutboxWorker =
    OutboxWorker(
        store = store,
        registry = registry,
        sender = sender,
        clock = clock,
        isNetwork = { it is FakeNetworkFailure },
        dispatcher = StandardTestDispatcher(testScheduler),
    ).also { worker -> backgroundScope.coroutineContext.job.invokeOnCompletion { worker.stop() } }

/**
 * Started and past startup, with the pass that startup requests already run, so the rows and
 * outcomes a test sets up afterwards are seen only by the drain it requests itself.
 */
fun TestScope.startedWorker(
    store: OfflineStore,
    sender: OutboxSender,
    registry: SessionRegistry,
    clock: () -> Long = { testScheduler.currentTime },
): OutboxWorker =
    testWorker(store, sender, registry, clock).also {
        it.start()
        it.markStartupReady()
        runCurrent()
    }
