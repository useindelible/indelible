package app.indelible.core.offline

import app.indelible.db.testOfflineDatabase
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertNull
import kotlin.test.assertTrue

class ScopePurgerTest {
    private fun store() = SqlDelightOfflineStore(testOfflineDatabase())

    @Test
    fun purgeRunsRowsThenFilesThenFinish() =
        runTest {
            val store = store()
            val scope = "scope"
            store.clientIdentity(scope)
            store.upsertCachedDocument(scope, cachedDocument())
            val calls = mutableListOf<String>()

            ScopePurger(store) { calls.add(it) }.purge(scope)

            assertEquals(listOf(scope), calls)
            assertNull(store.cachedDocument(scope, "doc_1"))
            assertTrue(store.scopesWithState().none { it.first == scope })
        }

    @Test
    fun purgeIsIdempotentWhenRepeated() =
        runTest {
            val store = store()
            val scope = "scope"
            store.clientIdentity(scope)
            val purger = ScopePurger(store)

            purger.purge(scope)
            purger.purge(scope)

            assertTrue(store.scopesWithState().none { it.first == scope })
        }

    @Test
    fun resumeInterruptedIsNoOpWithoutStoredScopes() =
        runTest {
            val store = store()

            ScopePurger(store).resumeInterrupted()

            assertTrue(store.scopesWithState().isEmpty())
        }

    @Test
    fun resumeInterruptedOnlyPurgesScopesWithPendingFlag() =
        runTest {
            val store = store()
            store.clientIdentity("clean")
            store.clientIdentity("pending")
            store.setPurgePending("pending", true)

            ScopePurger(store).resumeInterrupted()

            val remaining = store.scopesWithState().map { it.first }
            assertTrue("clean" in remaining)
            assertTrue("pending" !in remaining)
        }

    @Test
    fun resumeInterruptedRerunsFilesAfterCrashBeforeDeleteFilesCompletes() =
        runTest {
            val store = store()
            val scope = "scope"
            store.clientIdentity(scope)
            store.upsertCachedDocument(scope, cachedDocument())
            val calls = mutableListOf<String>()
            val throwingDeleteFiles: suspend (String) -> Unit = { s ->
                calls.add(s)
                if (calls.size == 1) error("simulated crash")
            }

            assertFailsWith<IllegalStateException> {
                ScopePurger(store, throwingDeleteFiles).purge(scope)
            }

            assertNull(store.cachedDocument(scope, "doc_1"))
            assertEquals(listOf(scope to true), store.scopesWithState())

            ScopePurger(store) { calls.add(it) }.resumeInterrupted()

            assertEquals(listOf(scope, scope), calls)
            assertTrue(store.scopesWithState().isEmpty())
        }

    @Test
    fun resumeInterruptedFinishesAfterCrashBetweenDeleteFilesAndFinishPurge() =
        runTest {
            val store = store()
            val scope = "scope"
            store.clientIdentity(scope)
            store.upsertCachedDocument(scope, cachedDocument())
            val calls = mutableListOf<String>()

            assertFailsWith<IllegalStateException> {
                ScopePurger(ThrowingFinishPurgeStore(store)) { calls.add(it) }.purge(scope)
            }

            assertEquals(listOf(scope), calls)
            assertNull(store.cachedDocument(scope, "doc_1"))
            assertEquals(listOf(scope to true), store.scopesWithState())

            ScopePurger(store) { calls.add(it) }.resumeInterrupted()

            assertEquals(listOf(scope, scope), calls)
            assertTrue(store.scopesWithState().isEmpty())
        }

    @Test
    fun resumeInterruptedContinuesPastAFailingScope() =
        runTest {
            val store = store()
            store.clientIdentity("broken")
            store.setPurgePending("broken", true)
            store.clientIdentity("ok")
            store.setPurgePending("ok", true)
            val purger =
                ScopePurger(store) { scope ->
                    if (scope == "broken") error("simulated failure")
                }

            purger.resumeInterrupted()

            val states = store.scopesWithState().toMap()
            assertEquals(mapOf("broken" to true), states)
        }

    @Test
    fun purgeInactivePurgesEveryScopeExceptActive() =
        runTest {
            val store = store()
            store.clientIdentity("scopeA")
            store.clientIdentity("scopeB")
            store.clientIdentity("scopeC")

            ScopePurger(store).purgeInactive("scopeB")

            val remaining = store.scopesWithState().map { it.first }
            assertEquals(listOf("scopeB"), remaining)
        }

    @Test
    fun purgeInactiveContinuesPastAFailingScope() =
        runTest {
            val store = store()
            store.clientIdentity("broken")
            store.clientIdentity("ok")
            val purger =
                ScopePurger(store) { scope ->
                    if (scope == "broken") error("simulated failure")
                }

            purger.purgeInactive(active = null)

            assertEquals(mapOf("broken" to true), store.scopesWithState().toMap())
        }

    @Test
    fun purgeInactiveIsNoOpWithoutStoredScopes() =
        runTest {
            val store = store()

            ScopePurger(store).purgeInactive("scopeA")

            assertTrue(store.scopesWithState().isEmpty())
        }

    @Test
    fun purgeInactivePurgesEveryScopeWhenActiveIsNull() =
        runTest {
            val store = store()
            store.clientIdentity("scopeA")
            store.clientIdentity("scopeB")

            ScopePurger(store).purgeInactive(null)

            assertTrue(store.scopesWithState().isEmpty())
        }

    private class ThrowingFinishPurgeStore(
        delegate: OfflineStore,
    ) : OfflineStore by delegate {
        override suspend fun finishPurge(scope: String): Unit = error("crash")
    }

    private fun cachedDocument() =
        CachedDocumentRow(
            "doc_1",
            "article",
            "Title",
            "{}",
            pinned = false,
            lastOpenedAt = 1L,
            lastSyncedAt = null,
            bytes = 5L,
        )
}
