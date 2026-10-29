package app.indelible.core.offline

import kotlinx.coroutines.flow.first
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertNull
import kotlin.test.assertTrue

class ListSnapshotStoreTest {
    private fun row(
        documentId: String,
        itemId: String = documentId,
        type: String = "article",
    ) = SnapshotRow(itemId, documentId, type, "Title $documentId", """{"id":"$itemId"}""")

    @Test
    fun replacingAListKeepsOnlyTheNewRowsInOrder() =
        runTest {
            val signedIn = signedInTestStore()
            val store = signedIn.store
            store.replaceList(signedIn.session, "home:recent", listOf(row("a"), row("b")), at = 1)

            store.replaceList(signedIn.session, "home:recent", listOf(row("c"), row("a")), at = 2)

            assertEquals(listOf(row("c"), row("a")), store.list(signedIn.scope, "home:recent"))
        }

    @Test
    fun appendingContinuesAfterTheLastPosition() =
        runTest {
            val signedIn = signedInTestStore()
            val store = signedIn.store
            store.replaceList(signedIn.session, "triage:inbox:all", listOf(row("a"), row("b")), at = 1)

            store.appendList(signedIn.session, "triage:inbox:all", listOf(row("c")), at = 2)

            assertEquals(listOf("a", "b", "c"), store.list(signedIn.scope, "triage:inbox:all").map { it.documentId })
        }

    @Test
    fun listsAreKeptPerKeyAndPerScope() =
        runTest {
            val signedIn = signedInTestStore("origin|a")
            val store = signedIn.store
            store.replaceList(signedIn.session, "home:recent", listOf(row("a")), at = 1)
            store.replaceList(signedIn.session, "home:continue", listOf(row("b")), at = 1)
            val other = signedIn.switchTo("origin|b")

            store.replaceList(other, "home:recent", listOf(row("c")), at = 1)

            assertEquals(listOf(row("a")), store.list("origin|a", "home:recent"))
            assertEquals(listOf(row("b")), store.list("origin|a", "home:continue"))
            assertEquals(listOf(row("c")), store.list("origin|b", "home:recent"))
        }

    @Test
    fun observedListFollowsAReplace() =
        runTest {
            val signedIn = signedInTestStore()
            val store = signedIn.store
            store.replaceList(signedIn.session, "home:recent", listOf(row("a")), at = 1)
            assertEquals(KeptList(listOf(row("a")), 1), store.observeList(signedIn.scope, "home:recent").first())

            store.replaceList(signedIn.session, "home:recent", listOf(row("b")), at = 2)

            assertEquals(KeptList(listOf(row("b")), 2), store.observeList(signedIn.scope, "home:recent").first())
        }

    @Test
    fun lastSeenIsTheNewestRowForTheDocumentInAnyList() =
        runTest {
            val signedIn = signedInTestStore()
            val store = signedIn.store
            store.replaceList(signedIn.session, "home:recent", listOf(row("doc_1", type = "pdf")), at = 1)
            store.replaceList(signedIn.session, "home:continue", listOf(row("doc_1", type = "book")), at = 5)

            assertEquals("book", store.lastSeen(signedIn.scope, "doc_1")?.documentType)
            assertNull(store.lastSeen(signedIn.scope, "doc_unknown"))
        }

    @Test
    fun lastSeenPrefersALibraryRowOverANewerHomeRow() =
        runTest {
            val signedIn = signedInTestStore()
            val store = signedIn.store
            store.replaceList(signedIn.session, "triage:inbox:all", listOf(row("doc_1", itemId = "entry_1")), at = 1)
            store.replaceList(signedIn.session, "home:recent", listOf(row("doc_1")), at = 9)

            assertEquals("entry_1", store.lastSeen(signedIn.scope, "doc_1")?.itemId)
        }

    @Test
    fun forgettingByARowIdDropsTheDocumentFromEveryList() =
        runTest {
            val signedIn = signedInTestStore()
            val store = signedIn.store
            store.replaceList(
                signedIn.session,
                "triage:inbox:all",
                listOf(row("doc_1", itemId = "entry_1"), row("doc_2", itemId = "entry_2")),
                at = 1,
            )
            store.replaceList(signedIn.session, "home:recent", listOf(row("doc_1")), at = 1)

            store.forget(signedIn.session, "entry_1")

            assertEquals(listOf("doc_2"), store.list(signedIn.scope, "triage:inbox:all").map { it.documentId })
            assertTrue(store.list(signedIn.scope, "home:recent").isEmpty())
        }

    @Test
    fun forgettingKeepsTheListsTheCallerKeeps() =
        runTest {
            val signedIn = signedInTestStore()
            val store = signedIn.store
            val keys = listOf("triage:inbox:all", "triage:all:pdf", "triage:later:all", "home:recent")
            keys.forEach { store.replaceList(signedIn.session, it, listOf(row("doc_1", itemId = "entry_1")), at = 1) }

            store.forget(signedIn.session, "entry_1") { key ->
                !key.startsWith("triage:") || key.startsWith("triage:all:") || key.startsWith("triage:later:")
            }

            val left = keys.filter { store.list(signedIn.scope, it).isNotEmpty() }
            assertEquals(listOf("triage:all:pdf", "triage:later:all", "home:recent"), left)
        }

    @Test
    fun aStaleSessionWritesNothing() =
        runTest {
            val signedIn = signedInTestStore("origin|a")
            val stale = signedIn.session
            signedIn.switchTo("origin|a")

            assertFailsWith<StaleWriteException> {
                signedIn.store.replaceList(stale, "home:recent", listOf(row("a")), at = 1)
            }
            assertTrue(signedIn.store.list("origin|a", "home:recent").isEmpty())
        }
}
