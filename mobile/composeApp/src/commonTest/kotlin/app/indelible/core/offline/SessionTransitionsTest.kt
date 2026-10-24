package app.indelible.core.offline

import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.async
import kotlinx.coroutines.awaitCancellation
import kotlinx.coroutines.cancelAndJoin
import kotlinx.coroutines.launch
import kotlinx.coroutines.test.runCurrent
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertFalse
import kotlin.test.assertNotEquals
import kotlin.test.assertNotNull
import kotlin.test.assertNull
import kotlin.test.assertSame
import kotlin.test.assertTrue

class SessionTransitionsTest {
    @Test
    fun transitionPublishesEpochPlusOneAndTheStoredSessionAfterTheBlock() =
        runTest {
            val h = transitionsHarness()
            h.tokenStorage.saveServerUrl("http://a")

            val result =
                h.sessions.transition(0) {
                    h.tokenStorage.saveToken("t")
                    h.tokenStorage.saveUserId("u1")
                }

            assertEquals(TransitionResult.Applied, result)
            val state = h.registry.current.value
            assertEquals(1L, state.epoch)
            assertEquals("http://a|u1", state.session?.scope)
            assertEquals("http://a", state.session?.origin)
            assertEquals(listOf("http://a|u1" to false), h.store.scopesWithState())
        }

    @Test
    fun staleExpectedEpochIsRejectedBeforeAnySideEffect() =
        runTest {
            val h = transitionsHarness()
            val b = h.signIn("http://a", "u1")
            var ran = false

            val result = h.sessions.transition(0) { ran = true }

            assertEquals(TransitionResult.Rejected, result)
            assertFalse(ran)
            assertSame(b, h.registry.current.value.session)
            assertEquals(1L, h.registry.current.value.epoch)
        }

    @Test
    fun transitionStartedWhileASendIsSuspendedPublishesNothingUntilThePassEnds() =
        runTest {
            val h = transitionsHarness()
            val a = h.signIn("http://a", "u1")
            h.store.enqueueNote(a, "doc_1")
            val started = h.sender.holdNextSend()
            h.worker.requestDrain()
            started.await()

            val logout = launch { h.sessions.transition(a.epoch) { h.tokenStorage.clearAll() } }
            runCurrent()

            assertEquals(SessionState(a.epoch + 1, null, transitioning = true), h.registry.current.value)
            assertFalse(logout.isCompleted)

            h.sender.release()
            logout.join()

            assertEquals(SessionState(a.epoch + 1, null), h.registry.current.value)
        }

    @Test
    fun twoConcurrentTransitionsSerializeAndTheSecondIsRejectedByItsStaleEpoch() =
        runTest {
            val h = transitionsHarness()
            h.tokenStorage.saveServerUrl("http://a")

            val first =
                async {
                    h.sessions.transition(0) {
                        h.tokenStorage.saveToken("t")
                        h.tokenStorage.saveUserId("u1")
                    }
                }
            val second = async { h.sessions.transition(0) { h.tokenStorage.saveUserId("u2") } }
            runCurrent()

            assertEquals(TransitionResult.Applied, first.await())
            assertEquals(TransitionResult.Rejected, second.await())
            assertEquals(1L, h.registry.current.value.epoch)
            assertEquals("http://a|u1", h.currentScope())
        }

    @Test
    fun writerHoldingTheOldSessionFailsAfterASameAccountReloginWhileANewWriteSucceeds() =
        runTest {
            val h = transitionsHarness()
            val a = h.signIn("http://a", "u1")

            h.signIn("http://a", "u1", expectedEpoch = a.epoch)

            assertFailsWith<StaleWriteException> { h.store.enqueueNote(a, "doc_1") }
            val b = checkNotNull(h.registry.current.value.session)
            h.store.enqueueNote(b, "doc_1")
            assertEquals(1, h.store.pendingOrdered(b.scope).size)
        }

    @Test
    fun transitionAdvancesTheEpochWhenItOpensSoAMidTransitionRefreshIsDiscarded() =
        runTest {
            val h = transitionsHarness()
            val a = h.signIn("http://a", "u1")
            val refreshWrites = mutableListOf<Boolean>()

            h.sessions.transition(a.epoch) {
                assertEquals(SessionState(a.epoch + 1, null, transitioning = true), h.registry.current.value)
                for (epoch in listOf(a.epoch, a.epoch + 1)) {
                    refreshWrites += h.registry.publishRefreshed(epoch) { h.tokenStorage.saveToken("late-a") }
                }
                h.tokenStorage.saveToken("token-b")
                h.tokenStorage.saveUserId("u2")
            }

            assertEquals(listOf(false, false), refreshWrites)
            assertEquals("token-b", h.tokenStorage.getToken())
            assertEquals(a.epoch + 1, h.registry.current.value.epoch)
            assertFalse(h.registry.current.value.transitioning)
        }

    @Test
    fun cancelledTransitionStillClosesAndReleasesTheWorker() =
        runTest {
            val h = transitionsHarness()
            val a = h.signIn("http://a", "u1")
            val blockEntered = CompletableDeferred<Unit>()
            val transition =
                launch {
                    h.sessions.transition(a.epoch) {
                        blockEntered.complete(Unit)
                        awaitCancellation()
                    }
                }
            blockEntered.await()

            transition.cancelAndJoin()

            assertEquals(SessionState(a.epoch + 1, null), h.registry.current.value)
            assertNotNull(h.signIn("http://a", "u1").scope)
        }

    @Test
    fun blockThatFailsAfterAPartialCredentialWriteLeavesNoSessionAndNoCredentials() =
        runTest {
            val h = transitionsHarness()
            val a = h.signIn("http://a", "u1")

            assertFailsWith<IllegalStateException> {
                h.sessions.transition(a.epoch) {
                    h.tokenStorage.saveToken("token-u2")
                    error("purge failed")
                }
            }

            assertEquals(SessionState(a.epoch + 1, null), h.registry.current.value)
            assertNull(h.tokenStorage.getToken())
            assertNull(h.tokenStorage.getUserId())
            h.signIn("http://a", "u2")
        }

    @Test
    fun blockCancelledAfterAPartialCredentialWriteLeavesNoSession() =
        runTest {
            val h = transitionsHarness()
            val a = h.signIn("http://a", "u1")
            val entered = CompletableDeferred<Unit>()
            val transition =
                launch {
                    h.sessions.transition(a.epoch) {
                        h.tokenStorage.saveToken("token-u2")
                        entered.complete(Unit)
                        awaitCancellation()
                    }
                }
            entered.await()

            transition.cancelAndJoin()

            assertEquals(SessionState(a.epoch + 1, null), h.registry.current.value)
            assertNull(h.tokenStorage.getToken())
        }

    @Test
    fun cancellationWhileJoiningThePassStillReleasesTheWorker() =
        runTest {
            val h = transitionsHarness()
            val a = h.signIn("http://a", "u1")
            h.store.enqueueNote(a, "doc_1")
            val started = h.sender.holdNextSend()
            h.worker.requestDrain()
            started.await()
            val transition = launch { h.sessions.transition(a.epoch) { } }
            runCurrent()

            transition.cancelAndJoin()
            h.sender.release()
            runCurrent()

            assertFalse(h.registry.current.value.transitioning)
            val b = h.signIn("http://a", "u2")
            h.store.enqueueNote(b, "doc_2")
            h.worker.requestDrain()
            runCurrent()
            assertEquals(2, h.sender.calls.size)
        }

    @Test
    fun identityCreationFailureClosesWithNoSession() =
        runTest {
            val h = transitionsHarness()
            h.tokenStorage.saveServerUrl("http://a")
            val sessions = SessionTransitions(h.registry, h.tokenStorage, FailingIdentityStore(h.store), h.worker)

            assertFailsWith<IllegalStateException> {
                sessions.transition(0) {
                    h.tokenStorage.saveToken("t")
                    h.tokenStorage.saveUserId("u1")
                }
            }

            assertEquals(SessionState(1, null), h.registry.current.value)
            assertEquals("http://a|u1", h.signIn("http://a", "u1").scope)
        }

    @Test
    fun refreshInFlightAcrossATransitionIsDiscarded() =
        runTest {
            val h = transitionsHarness()
            val a = h.signIn("http://a", "u1")

            h.signIn("http://a", "u2", expectedEpoch = a.epoch)

            assertFalse(h.registry.publishRefreshed(a.epoch) { h.tokenStorage.saveToken("late") })
            assertNotEquals("late", h.tokenStorage.getToken())
        }

    @Test
    fun blockFailurePropagatesButStillClosesAndReleasesTheWorker() =
        runTest {
            val h = transitionsHarness()
            h.tokenStorage.saveServerUrl("http://a")

            assertFailsWith<IllegalStateException> {
                h.sessions.transition(0) {
                    h.tokenStorage.saveToken("t")
                    h.tokenStorage.saveUserId("u1")
                    error("boom")
                }
            }

            assertEquals(SessionState(1, null), h.registry.current.value)
            assertEquals("http://a|u1", h.signIn("http://a", "u1").scope)
        }

    @Test
    fun restorePublishesTheStoredSessionAtEpochZeroAndCreatesClientState() =
        runTest {
            val h = transitionsHarness()
            h.tokenStorage.saveServerUrl("http://a")
            h.tokenStorage.saveToken("t")
            h.tokenStorage.saveUserId("u1")

            h.sessions.restore()

            assertEquals(0L, h.registry.current.value.epoch)
            assertEquals("http://a|u1", h.currentScope())
            assertEquals(listOf("http://a|u1" to false), h.store.scopesWithState())
        }

    @Test
    fun restoreWithTokensButNoUserIdPublishesNoSession() =
        runTest {
            val h = transitionsHarness()
            h.tokenStorage.saveServerUrl("http://a")
            h.tokenStorage.saveToken("t")

            h.sessions.restore()

            assertNull(h.registry.current.value.session)
            assertTrue(h.store.scopesWithState().isEmpty())
        }

    @Test
    fun restoreWithAUserIdButNoCredentialPublishesNoSession() =
        runTest {
            val h = transitionsHarness()
            h.tokenStorage.saveServerUrl("http://a")
            h.tokenStorage.saveUserId("u1")

            h.sessions.restore()

            assertNull(h.registry.current.value.session)
        }

    @Test
    fun restoreAfterASignInIsANoOp() =
        runTest {
            val h = transitionsHarness()
            val a = h.signIn("http://a", "u1")

            h.sessions.restore()

            assertSame(a, h.registry.current.value.session)
            assertEquals(1L, h.registry.current.value.epoch)
        }

    @Test
    fun enqueueDuringPurgeAndAfterFinishPurgeIsRejectedAsNotLive() =
        runTest {
            val h = transitionsHarness()
            val a = h.signIn("http://a", "u1")

            h.store.setPurgePending(a.scope, true)
            assertFailsWith<ScopeNotLiveException> { h.store.enqueueNote(a, "doc_1") }

            h.store.purgeRows(a.scope)
            h.store.finishPurge(a.scope)
            assertFailsWith<ScopeNotLiveException> { h.store.enqueueNote(a, "doc_1") }
            assertTrue(h.store.scopesWithState().isEmpty())
        }

    @Test
    fun transitionEndsWithADrainRequestThatRunsOnceTheWorkerIsReleased() =
        runTest {
            val h = transitionsHarness()
            val a = h.signIn("http://a", "u1")
            h.store.enqueueNote(a, "doc_1")

            h.signIn("http://a", "u1", expectedEpoch = a.epoch)
            runCurrent()

            assertEquals(1, h.sender.calls.size)
        }
}
