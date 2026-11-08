package app.indelible.reader.viewmodel

import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.advanceTimeBy
import kotlinx.coroutines.test.runCurrent
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals

@OptIn(ExperimentalCoroutinesApi::class)
class ReaderReconcileTest {
    private val online = MutableStateFlow(true)
    private val reachable = MutableStateFlow<Boolean?>(true)
    private val changes = MutableSharedFlow<String>(extraBufferCapacity = 16)
    private var refreshes = 0
    private var answers = true

    private fun CoroutineScope.start() =
        reconcileOn(readerTriggers("doc_1", ReaderTriggers(online, reachable, changes))) {
            refreshes++
            answers
        }

    @Test
    fun theFirstSamplesAreABaseline() =
        runTest {
            val job = start()
            runCurrent()

            assertEquals(0, refreshes)
            job.cancel()
        }

    @Test
    fun connectivityReturningRefreshesEvenWhenReachableNeverMoved() =
        runTest {
            val job = start()
            runCurrent()

            online.value = false
            runCurrent()
            online.value = true
            runCurrent()

            assertEquals(1, refreshes)
            job.cancel()
        }

    @Test
    fun aReachableRisingEdgeRefreshes() =
        runTest {
            val job = start()
            runCurrent()

            reachable.value = false
            runCurrent()
            reachable.value = true
            runCurrent()

            assertEquals(1, refreshes)
            job.cancel()
        }

    @Test
    fun repeatedTrueDoesNotRefreshAgain() =
        runTest {
            val job = start()
            runCurrent()

            reachable.value = false
            runCurrent()
            reachable.value = true
            runCurrent()
            reachable.value = true
            runCurrent()
            online.value = true
            runCurrent()

            assertEquals(1, refreshes)
            job.cancel()
        }

    @Test
    fun aChangeForThisDocumentRefreshes() =
        runTest {
            val job = start()
            runCurrent()

            changes.tryEmit("doc_1")
            runCurrent()

            assertEquals(1, refreshes)
            job.cancel()
        }

    @Test
    fun aChangeForAnotherDocumentDoesNothing() =
        runTest {
            val job = start()
            runCurrent()

            changes.tryEmit("doc_2")
            runCurrent()

            assertEquals(0, refreshes)
            job.cancel()
        }

    @Test
    fun aFailedRefreshRetriesAtThirtySecondsThenTwoMinutesThenWaits() =
        runTest {
            answers = false
            val job = start()
            runCurrent()

            changes.tryEmit("doc_1")
            runCurrent()
            assertEquals(1, refreshes)

            advanceTimeBy(30_000L)
            runCurrent()
            assertEquals(2, refreshes)

            advanceTimeBy(120_000L)
            runCurrent()
            assertEquals(3, refreshes)

            advanceTimeBy(600_000L)
            runCurrent()
            assertEquals(3, refreshes)
            job.cancel()
        }

    @Test
    fun aNewTriggerDropsThePendingRetry() =
        runTest {
            answers = false
            val job = start()
            runCurrent()

            changes.tryEmit("doc_1")
            runCurrent()
            answers = true
            changes.tryEmit("doc_1")
            runCurrent()
            assertEquals(2, refreshes)

            advanceTimeBy(200_000L)
            runCurrent()
            assertEquals(2, refreshes)
            job.cancel()
        }

    @Test
    fun emptyFlowsNeverRefresh() =
        runTest {
            val job =
                reconcileOn(readerTriggers("doc_1", ReaderTriggers(reachable = flowOf(true)))) {
                    refreshes++
                    true
                }
            runCurrent()
            advanceTimeBy(600_000L)
            runCurrent()

            assertEquals(0, refreshes)
            job.cancel()
        }

    @Test
    fun aTriggerDuringARefreshJoinsIt() =
        runTest {
            val gate = CompletableDeferred<Unit>()
            val job =
                reconcileOn(readerTriggers("doc_1", ReaderTriggers(changes = changes))) {
                    refreshes++
                    gate.await()
                    true
                }
            runCurrent()

            changes.tryEmit("doc_1")
            runCurrent()
            changes.tryEmit("doc_1")
            runCurrent()
            gate.complete(Unit)
            runCurrent()

            assertEquals(1, refreshes)
            job.cancel()
        }
}
