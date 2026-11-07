package app.indelible.core.offline

import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNull
import kotlin.test.assertSame
import kotlin.test.assertTrue

class SessionRegistryTest {
    @Test
    fun startsAtEpochZeroWithNoSession() {
        assertEquals(SessionState(0, null), SessionRegistry().current.value)
    }

    @Test
    fun restorePublishesOnlyWhenNothingWasPublishedYet() =
        runTest {
            val registry = SessionRegistry()
            val a = testSession("http://a|u1")

            assertTrue(registry.restore(a))
            assertSame(a, registry.current.value.session)
            assertEquals(0L, registry.current.value.epoch)

            assertFalse(registry.restore(testSession("http://b|u2")))
            assertSame(a, registry.current.value.session)
        }

    @Test
    fun restoreIsRefusedOnceTheEpochHasMoved() =
        runTest {
            val registry = SessionRegistry()
            registry.publish(SessionState(1, null))

            assertFalse(registry.restore(testSession("http://a|u1")))
            assertNull(registry.current.value.session)
        }

    @Test
    fun publishRefreshedWritesOnlyWhenTheEpochStillMatches() =
        runTest {
            val registry = SessionRegistry()
            var writes = 0

            assertTrue(registry.publishRefreshed(0) { writes++ })
            registry.publish(SessionState(1, null))
            assertFalse(registry.publishRefreshed(0) { writes++ })

            assertEquals(1, writes)
        }

    @Test
    fun invalidateClearsTheSessionOnlyForItsOwnEpoch() =
        runTest {
            val registry = SessionRegistry()
            val b = testSession("http://b|u2", epoch = 1)
            registry.publish(SessionState(1, b))

            assertFalse(registry.invalidate(0))
            assertSame(b, registry.current.value.session)

            assertTrue(registry.invalidate(1))
            assertNull(registry.current.value.session)
            assertEquals(1L, registry.current.value.epoch)
        }

    @Test
    fun publishRefreshedIsRefusedWhileATransitionIsOpen() =
        runTest {
            val registry = SessionRegistry()
            registry.publish(SessionState(1, null, transitioning = true))
            var writes = 0

            assertFalse(registry.publishRefreshed(1) { writes++ })

            registry.publish(SessionState(1, testSession("http://a|u1", epoch = 1)))
            assertTrue(registry.publishRefreshed(1) { writes++ })
            assertEquals(1, writes)
        }

    @Test
    fun invalidateIsRefusedWhileATransitionIsOpen() =
        runTest {
            val registry = SessionRegistry()
            registry.publish(SessionState(1, null, transitioning = true))

            assertFalse(registry.invalidate(1))
            assertTrue(registry.current.value.transitioning)
        }

    @Test
    fun sessionsForTheSameAccountAreDistinctObjects() {
        val first = testSession("http://a|u1")
        val second = testSession("http://a|u1")

        assertFalse(first == second)
        assertEquals(first.scope, second.scope)
    }
}
