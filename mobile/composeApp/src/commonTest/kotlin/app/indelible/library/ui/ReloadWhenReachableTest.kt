package app.indelible.library.ui

import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.launch
import kotlinx.coroutines.test.TestScope
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals

class ReloadWhenReachableTest {
    private val online = MutableStateFlow(false)
    private val reachable = MutableStateFlow<Boolean?>(null)

    private fun TestScope.edges(stale: Boolean = false): MutableList<Unit> {
        val seen = mutableListOf<Unit>()
        backgroundScope.launch(UnconfinedTestDispatcher(testScheduler)) {
            reachableAgain(online, reachable, stale).collect { seen += it }
        }
        return seen
    }

    @Test
    fun theFirstValuesOnlySetTheBaseline() =
        runTest {
            online.value = true
            reachable.value = true

            assertEquals(0, edges().size)
        }

    @Test
    fun aStaleScreenReloadsAtOnceWhenTheServerAlreadyAnswers() =
        runTest {
            online.value = true
            reachable.value = true

            assertEquals(1, edges(stale = true).size)
        }

    @Test
    fun aStaleScreenWaitsForTheServerToAnswer() =
        runTest {
            online.value = true
            reachable.value = false
            val seen = edges(stale = true)
            assertEquals(0, seen.size)

            reachable.value = true

            assertEquals(1, seen.size)
        }

    @Test
    fun theNetworkComingBackFires() =
        runTest {
            val seen = edges()

            online.value = true

            assertEquals(1, seen.size)
        }

    @Test
    fun aRequestGettingThroughFires() =
        runTest {
            reachable.value = false
            val seen = edges()

            reachable.value = true

            assertEquals(1, seen.size)
        }

    @Test
    fun repeatsAreIgnored() =
        runTest {
            val seen = edges()

            online.value = true
            online.value = true
            reachable.value = false
            reachable.value = false

            assertEquals(1, seen.size)
        }
}
