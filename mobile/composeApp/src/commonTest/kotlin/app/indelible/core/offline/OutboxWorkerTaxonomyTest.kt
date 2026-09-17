package app.indelible.core.offline

import kotlinx.coroutines.test.runCurrent
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class OutboxWorkerTaxonomyTest {
    private val scope = "scope"

    @Test
    fun row4ConnectionDropMidDrainKeepsCommittedRowsAndStopsAtTheFailure() =
        runTest {
            val store = testStore()
            val sender = FakeOutboxSender()
            val clock = FakeClock(0L)
            val worker = startedWorker(store, sender, scope, clock::current)
            store.enqueueNote(scope, "doc_1")
            store.enqueueNote(scope, "doc_2")
            val thirdId = store.enqueueNote(scope, "doc_3")
            val fourthId = store.enqueueNote(scope, "doc_4")
            sender.enqueueOutcome(SendOutcome.Success)
            sender.enqueueOutcome(SendOutcome.Success)
            sender.enqueueOutcome(SendOutcome.Transport(FakeNetworkFailure()))

            worker.requestDrain()
            runCurrent()

            assertEquals(3, sender.calls.size)
            val pendingRows = store.rowsByState(scope, OutboxState.PENDING)
            val pending = pendingRows.single { it.id == thirdId }
            assertEquals(1, pending.attempts)
            val untouched = pendingRows.single { it.id == fourthId }
            assertEquals(0, untouched.attempts)
        }

    @Test
    fun row5PostRefreshSuccessLeavesNoTraceAndDoesNotPauseAuth() =
        runTest {
            val store = testStore()
            val sender = FakeOutboxSender()
            val worker = startedWorker(store, sender, scope)
            store.enqueueNote(scope, "doc_1")
            sender.enqueueOutcome(SendOutcome.Success)

            worker.requestDrain()
            runCurrent()

            assertTrue(store.pendingOrdered(scope).isEmpty())
            assertEquals(false, worker.authPaused.value)
        }

    @Test
    fun row18FailedCreateBlocksDependantsAndRetryRowUnblocksThem() =
        runTest {
            val store = testStore()
            val sender = FakeOutboxSender()
            val worker = startedWorker(store, sender, scope)
            val createId = store.enqueueHighlightCreate(scope, "hlt_1", "doc_1")
            val colorId = store.enqueueHighlightColor(scope, "hlt_1", "doc_1")
            sender.enqueueOutcome(SendOutcome.Http(422, null, "anchor invalid"))

            worker.requestDrain()
            runCurrent()

            assertEquals(createId, store.rowsByState(scope, OutboxState.FAILED).single().id)
            assertEquals(colorId, store.rowsByState(scope, OutboxState.BLOCKED).single().id)

            store.retryRow(scope, createId)

            assertTrue(store.rowsByState(scope, OutboxState.BLOCKED).isEmpty())
            assertEquals(
                setOf(createId, colorId),
                store.rowsByState(scope, OutboxState.PENDING).map { it.id }.toSet(),
            )
        }

    @Test
    fun row19DeleteReplayedAs404IsTreatedAsSuccess() =
        runTest {
            val store = testStore()
            val sender = FakeOutboxSender()
            val worker = startedWorker(store, sender, scope)
            store.enqueueHighlightDelete(scope, "hlt_1", "doc_1")
            sender.enqueueOutcome(SendOutcome.ReplaySuccess)

            worker.requestDrain()
            runCurrent()

            assertTrue(store.pendingOrdered(scope).isEmpty())
        }

    @Test
    fun row22RowsLeftPendingAreDrainedByANewWorkerInstance() =
        runTest {
            val store = testStore()
            val firstSender = FakeOutboxSender()
            val firstWorker = startedWorker(store, firstSender, scope)
            store.enqueueNote(scope, "doc_1")
            firstSender.enqueueOutcome(SendOutcome.Transport(FakeNetworkFailure()))
            firstWorker.requestDrain()
            runCurrent()
            assertEquals(1, store.rowsByState(scope, OutboxState.PENDING).size)

            val secondSender = FakeOutboxSender()
            secondSender.enqueueOutcome(SendOutcome.Success)
            startedWorker(store, secondSender, scope) { Long.MAX_VALUE }

            assertTrue(store.pendingOrdered(scope).isEmpty())
            assertEquals(1, secondSender.calls.size)
        }

    @Test
    fun row23RecordedAtPassesThroughUnclampedToTheSender() =
        runTest {
            val store = testStore()
            val sender = FakeOutboxSender()
            val worker = startedWorker(store, sender, scope)
            val skewedRecordedAt = 9_999_999_999_999L
            store.enqueueReadingEvent(scope, "doc_1", skewedRecordedAt)
            sender.enqueueOutcome(SendOutcome.Success)

            worker.requestDrain()
            runCurrent()

            val sentPayload =
                sender.calls
                    .single()
                    .single()
                    .payload as OutboxPayload.ReadingEvent
            assertEquals(skewedRecordedAt, sentPayload.recordedAtEpochMs)
        }

    @Test
    fun row24LargeBacklogBatchesReadingEventsInSeqOrder() =
        runTest {
            val store = testStore()
            val spyStore = MarkDocumentSyncedSpyStore(store)
            val sender = FakeOutboxSender()
            val worker = startedWorker(spyStore, sender, scope)
            repeat(250) { store.enqueueReadingEvent(scope, "doc_1", it.toLong()) }
            repeat(2) { sender.enqueueOutcome(SendOutcome.Success) }

            worker.requestDrain()
            runCurrent()

            assertEquals(2, sender.calls.size)
            assertEquals(200, sender.calls[0].size)
            assertEquals(50, sender.calls[1].size)
            val allSeqs = sender.calls.flatten().map { it.seq }
            assertEquals(allSeqs.sorted(), allSeqs)
            // One sync stamp per batch, not per row: 250 rows across 2 batches must not cost
            // 250 serialized store writes.
            assertEquals(2, spyStore.markDocumentSyncedCallCount)
        }

    @Test
    fun row14RetryAfterPersistsToTheStoreAndStopsBeforeTheNextEntity() =
        runTest {
            val store = testStore()
            val sender = FakeOutboxSender()
            val clock = FakeClock(1_000L)
            val worker = startedWorker(store, sender, scope, clock::current)
            val rateLimitedId = store.enqueueNote(scope, "doc_1")
            val followingId = store.enqueueNote(scope, "doc_2")
            sender.enqueueOutcome(SendOutcome.Http(429, retryAfterSeconds = 7, message = "slow down"))

            worker.requestDrain()
            runCurrent()

            assertEquals(1, sender.calls.size)
            val pendingRows = store.rowsByState(scope, OutboxState.PENDING)
            val retried = pendingRows.single { it.id == rateLimitedId }
            assertEquals(1_000L + 7_000L, retried.nextAttemptAt)
            assertEquals(1, retried.attempts)
            assertEquals("HTTP 429: slow down", retried.lastError)
            val untouched = pendingRows.single { it.id == followingId }
            assertEquals(0, untouched.attempts)
        }
}
