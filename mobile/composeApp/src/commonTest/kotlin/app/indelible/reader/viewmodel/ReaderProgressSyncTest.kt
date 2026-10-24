package app.indelible.reader.viewmodel

import kotlinx.coroutines.CoroutineExceptionHandler
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals

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
                ReaderProgressSync(scope) { percent ->
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
}
