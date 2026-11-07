package app.indelible.home.ui.offline

import androidx.compose.runtime.State
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.runComposeUiTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

@OptIn(ExperimentalTestApi::class)
class SceneClockTest {
    @Test
    fun theSceneMovesForAWhileThenHolds() =
        runComposeUiTest {
            mainClock.autoAdvance = false
            lateinit var clock: State<Float>
            setContent { clock = rememberSceneClock(key = "offline") }

            mainClock.advanceTimeBy(SCENE_MOTION_MS / 2L)
            assertTrue(clock.value in 1f..<SCENE_MOTION_MS.toFloat())
            mainClock.advanceTimeBy(SCENE_MOTION_MS * 3L)
            val settled = clock.value
            mainClock.advanceTimeBy(SCENE_MOTION_MS * 3L)

            assertEquals(settled, clock.value)
        }

    @Test
    fun aNewStateMovesTheSceneAgainFromWhereItStopped() =
        runComposeUiTest {
            mainClock.autoAdvance = false
            var key by mutableStateOf("offline")
            lateinit var clock: State<Float>
            setContent { clock = rememberSceneClock(key) }
            mainClock.advanceTimeBy(SCENE_MOTION_MS * 2L)
            val stopped = clock.value

            key = "sending"
            mainClock.advanceTimeBy(SCENE_MOTION_MS * 3L)

            assertTrue(clock.value > stopped)
        }

    @Test
    fun phasesAndTwinkleStayInRange() {
        for (t in listOf(0f, 1_234f, 9_999f, 25_000f)) {
            assertTrue(phase(t, 1_600) in 0f..<1f)
            assertTrue(twinkle(t, 3_800) in 0.25f..1f)
        }
    }
}
