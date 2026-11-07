package app.indelible.reader.viewmodel

import app.indelible.core.offline.ChapterPosition
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.CoroutineExceptionHandler
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import kotlinx.coroutines.launch
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.runCurrent
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue

@OptIn(ExperimentalCoroutinesApi::class)
class ReaderProgressSyncTest {
    @Test
    fun a_callback_that_throws_does_not_stop_later_progress_from_flushing() =
        runTest {
            val flushed = mutableListOf<Float>()
            var failNext = true
            // The callback's failure is the subject here, so it is dropped rather than failing the test.
            val scope =
                CoroutineScope(
                    StandardTestDispatcher(testScheduler) + SupervisorJob() + CoroutineExceptionHandler { _, _ -> },
                )
            val sync =
                ReaderProgressSync(scope) { percent, _ ->
                    flushed += percent
                    if (failNext) {
                        failNext = false
                        error("local write failed")
                    }
                }

            sync.schedule(10f)
            advanceUntilIdle()
            sync.schedule(20f)
            sync.flush()
            advanceUntilIdle()

            assertEquals(listOf(10f, 20f), flushed)
            scope.cancel()
        }

    @Test
    fun flush_now_returns_after_the_enqueue() =
        runTest {
            val flushed = mutableListOf<Pair<Float, ChapterPosition?>>()
            val scope = CoroutineScope(StandardTestDispatcher(testScheduler) + SupervisorJob())
            val sync = ReaderProgressSync(scope) { percent, position -> flushed += percent to position }
            val position = ChapterPosition("ch2", 7)

            sync.schedule(10f, null)
            sync.schedule(20f, position)
            sync.flushNow()

            assertEquals(listOf(10f to null, 20f to position), flushed)
            scope.cancel()
        }

    @Test
    fun a_sample_queued_behind_a_failing_write_still_flushes() =
        runTest {
            val flushed = mutableListOf<Float>()
            val gate = CompletableDeferred<Unit>()
            val scope =
                CoroutineScope(
                    StandardTestDispatcher(testScheduler) + SupervisorJob() + CoroutineExceptionHandler { _, _ -> },
                )
            val sync =
                ReaderProgressSync(scope) { percent, _ ->
                    flushed += percent
                    if (flushed.size == 1) {
                        gate.await()
                        error("local write failed")
                    }
                }

            sync.schedule(10f)
            runCurrent()
            sync.schedule(20f)
            sync.flush()
            gate.complete(Unit)
            advanceUntilIdle()

            assertEquals(listOf(10f, 20f), flushed)
            scope.cancel()
        }

    @Test
    fun flush_now_waits_for_a_write_chained_on_an_immediate_dispatcher() =
        runTest {
            val gate = CompletableDeferred<Unit>()
            val scope = CoroutineScope(UnconfinedTestDispatcher(testScheduler) + SupervisorJob())
            lateinit var sync: ReaderProgressSync
            sync =
                ReaderProgressSync(scope) { percent, _ ->
                    if (percent == 10f) sync.schedule(20f) else gate.await()
                }
            sync.schedule(10f)

            val flushing = scope.launch { sync.flushNow() }

            assertFalse(flushing.isCompleted)
            gate.complete(Unit)
            assertTrue(flushing.isCompleted)
            scope.cancel()
        }
}
