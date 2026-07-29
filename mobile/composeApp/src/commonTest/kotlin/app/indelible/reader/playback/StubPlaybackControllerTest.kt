package app.indelible.reader.playback

import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.test.advanceTimeBy
import kotlinx.coroutines.test.runCurrent
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue

@OptIn(ExperimentalCoroutinesApi::class)
class StubPlaybackControllerTest {
    @Test
    fun play_flips_playing_and_advances_position() =
        runTest {
            val controller = StubPlaybackController(backgroundScope)

            controller.play()
            assertTrue(controller.state.value.isPlaying)

            advanceTimeBy(2_000)
            runCurrent()

            assertTrue(controller.state.value.positionMs > 0)
        }

    @Test
    fun pause_freezes_position() =
        runTest {
            val controller = StubPlaybackController(backgroundScope)

            controller.play()
            advanceTimeBy(1_000)
            runCurrent()
            controller.pause()
            val frozen = controller.state.value.positionMs

            assertFalse(controller.state.value.isPlaying)

            advanceTimeBy(3_000)
            runCurrent()

            assertEquals(frozen, controller.state.value.positionMs)
        }

    @Test
    fun skip_moves_position_forward() =
        runTest {
            val controller = StubPlaybackController(backgroundScope)

            controller.skip(15_000)

            assertEquals(15_000L, controller.state.value.positionMs)
        }

    @Test
    fun reaching_duration_stops_playback() =
        runTest {
            val controller = StubPlaybackController(backgroundScope)

            controller.seekTo(controller.state.value.durationMs - 500)
            controller.play()
            advanceTimeBy(3_000)
            runCurrent()

            assertEquals(controller.state.value.durationMs, controller.state.value.positionMs)
            assertFalse(controller.state.value.isPlaying)
        }

    @Test
    fun sleep_timer_pauses_after_elapsed_minutes() =
        runTest {
            val controller = StubPlaybackController(backgroundScope)

            controller.play()
            controller.setSleepTimer(1)
            advanceTimeBy(61_000)
            runCurrent()

            assertFalse(controller.state.value.isPlaying)
            assertEquals(null, controller.state.value.sleepTimerMinutes)
        }
}
