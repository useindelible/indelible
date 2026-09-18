package app.indelible.core.offline

import app.indelible.db.testOfflineDatabase
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNull
import kotlin.test.assertTrue

class OfflineStorePurgeTest {
    private fun store() = SqlDelightOfflineStore(testOfflineDatabase())

    @Test
    fun clientIdentityLazilyCreatesAndIsStableAcrossCalls() =
        runTest {
            val store = store()
            val scope = "scope"

            val first = store.clientIdentity(scope)
            val second = store.clientIdentity(scope)

            assertTrue(first.clientId.startsWith("cli_"))
            assertFalse(first.purgePending)
            assertEquals(first, second)
        }

    @Test
    fun clientIdentityIsIsolatedPerScope() =
        runTest {
            val store = store()

            val scopeA = store.clientIdentity("scopeA")
            val scopeB = store.clientIdentity("scopeB")

            assertTrue(scopeA.clientId != scopeB.clientId)
        }

    @Test
    fun setPurgePendingIsReflectedInClientIdentityAndScopesWithState() =
        runTest {
            val store = store()
            val scope = "scope"
            store.clientIdentity(scope)

            store.setPurgePending(scope, true)

            assertTrue(store.clientIdentity(scope).purgePending)
            assertEquals(listOf(scope to true), store.scopesWithState())
        }

    @Test
    fun scopesWithStateReturnsEveryKnownScope() =
        runTest {
            val store = store()
            store.clientIdentity("scopeA")
            store.clientIdentity("scopeB")
            store.setPurgePending("scopeB", true)

            val states = store.scopesWithState().toMap()

            assertEquals(mapOf("scopeA" to false, "scopeB" to true), states)
        }

    @Test
    fun purgeRowsRemovesCachedAndOutboxRowsButLeavesClientState() =
        runTest {
            val signedIn = signedInStore()
            val store = signedIn.store
            val session = signedIn.session
            val scope = signedIn.scope
            store.upsertCachedDocument(
                scope,
                CachedDocumentRow(
                    "doc_1",
                    "article",
                    "Title",
                    "{}",
                    pinned = false,
                    lastOpenedAt = 1L,
                    lastSyncedAt = null,
                    bytes = 5L,
                ),
            )
            store.enqueue(session, OutboxKind.DOCUMENT_NOTE, "doc_1", "doc_1") {
                OutboxPayload.DocumentNote("note", null) to Unit
            }

            store.purgeRows(scope)

            assertNull(store.cachedDocument(scope, "doc_1"))
            assertTrue(store.pendingOrdered(scope).isEmpty())
            assertEquals(listOf(scope to false), store.scopesWithState())
        }

    @Test
    fun finishPurgeRemovesTheClientStateRow() =
        runTest {
            val store = store()
            val scope = "scope"
            store.clientIdentity(scope)
            store.purgeRows(scope)

            store.finishPurge(scope)

            assertTrue(store.scopesWithState().isEmpty())
        }

    @Test
    fun purgeAndClientStateAreIsolatedByScope() =
        runTest {
            val store = store()
            store.clientIdentity("scopeA")
            store.clientIdentity("scopeB")
            store.upsertCachedDocument(
                "scopeA",
                CachedDocumentRow(
                    "doc_1",
                    "article",
                    "Title",
                    "{}",
                    pinned = false,
                    lastOpenedAt = 1L,
                    lastSyncedAt = null,
                    bytes = 5L,
                ),
            )

            store.purgeRows("scopeA")
            store.finishPurge("scopeA")

            val remainingScopes = store.scopesWithState().map { it.first }
            assertFalse("scopeA" in remainingScopes)
            assertTrue("scopeB" in remainingScopes)
        }
}
