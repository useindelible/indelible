package app.indelible.core.offline

import app.indelible.db.testOfflineDatabase
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFails
import kotlin.test.assertNull
import kotlin.test.assertTrue

class OfflineStoreCatalogTest {
    private fun store() = SqlDelightOfflineStore(testOfflineDatabase())

    private fun documentRow(
        id: String,
        pinned: Boolean = false,
        lastOpenedAt: Long = 1L,
        lastSyncedAt: Long? = null,
        bytes: Long = 10L,
    ) = CachedDocumentRow(
        documentId = id,
        documentType = "article",
        title = "Title $id",
        readerJson = "{}",
        pinned = pinned,
        lastOpenedAt = lastOpenedAt,
        lastSyncedAt = lastSyncedAt,
        bytes = bytes,
    )

    @Test
    fun upsertAndReadCachedDocumentRoundTrips() =
        runTest {
            val store = store()
            val scope = "scope"

            store.upsertCachedDocument(scope, documentRow("doc_1", bytes = 42L))

            val loaded = store.cachedDocument(scope, "doc_1")
            assertEquals("Title doc_1", loaded?.title)
            assertEquals(42L, loaded?.bytes)
        }

    @Test
    fun reUpsertingDocumentBuiltFromExistingRowPreservesLastSyncedAt() =
        runTest {
            val store = store()
            val scope = "scope"
            store.upsertCachedDocument(scope, documentRow("doc_1", lastSyncedAt = 5L))

            val existing = store.cachedDocument(scope, "doc_1")!!
            assertEquals(5L, existing.lastSyncedAt)

            store.upsertCachedDocument(scope, existing.copy(title = "Renamed"))

            val after = store.cachedDocument(scope, "doc_1")!!
            assertEquals(5L, after.lastSyncedAt)
            assertEquals("Renamed", after.title)
        }

    @Test
    fun touchSetPinnedAndSetBytesMutateTargetedColumns() =
        runTest {
            val store = store()
            val scope = "scope"
            store.upsertCachedDocument(scope, documentRow("doc_1"))

            store.touchDocumentOpened(scope, "doc_1", at = 99L)
            store.setPinned(scope, "doc_1", pinned = true)
            store.setDocumentBytes(scope, "doc_1", bytes = 500L)

            val row = store.cachedDocument(scope, "doc_1")!!
            assertEquals(99L, row.lastOpenedAt)
            assertTrue(row.pinned)
            assertEquals(500L, row.bytes)
        }

    @Test
    fun unpinnedLruOrdersByLastOpenedAtAscendingAndExcludesPinned() =
        runTest {
            val store = store()
            val scope = "scope"
            store.upsertCachedDocument(scope, documentRow("doc_old", lastOpenedAt = 1L))
            store.upsertCachedDocument(scope, documentRow("doc_new", lastOpenedAt = 2L))
            store.upsertCachedDocument(scope, documentRow("doc_pinned", pinned = true, lastOpenedAt = 0L))

            val lru = store.unpinnedLru(scope)

            assertEquals(listOf("doc_old", "doc_new"), lru.map { it.documentId })
        }

    @Test
    fun totalBytesSumsAllCachedDocumentsInScope() =
        runTest {
            val store = store()
            val scope = "scope"
            store.upsertCachedDocument(scope, documentRow("doc_1", bytes = 10L))
            store.upsertCachedDocument(scope, documentRow("doc_2", bytes = 20L))

            assertEquals(30L, store.totalBytes(scope))
            assertEquals(0L, store.totalBytes("empty-scope"))
        }

    @Test
    fun installCachedDocumentSwapsDocumentAndAssetsInOneTransaction() =
        runTest {
            val s = signedInStore()

            s.store.installCachedDocument(
                s.session,
                installRequest(s.revision(), assets = listOf(CachedAssetRow(VIEW_DOC, "html", 0, "v1.html", 10L))),
            )
            s.store.installCachedDocument(
                s.session,
                installRequest(
                    s.revision(),
                    generation = 2,
                    assets =
                        listOf(
                            CachedAssetRow(VIEW_DOC, "html", 0, "v2.html", 15L),
                            CachedAssetRow(VIEW_DOC, "img", 0, "v2.png", 5L),
                        ),
                ),
            )

            assertEquals(20L, s.store.cachedDocument(s.scope, VIEW_DOC)?.bytes)
            val assets = s.store.assetsForDocument(s.scope, VIEW_DOC)
            assertEquals(setOf("v2.html", "v2.png"), assets.map { it.path }.toSet())
        }

    @Test
    fun installCachedDocumentFailureMidListLeavesPreviousInstallIntact() =
        runTest {
            val s = signedInStore()
            s.store.installCachedDocument(
                s.session,
                installRequest(s.revision(), assets = listOf(CachedAssetRow(VIEW_DOC, "html", 0, "orig.html", 100L))),
            )

            val badAssets =
                listOf(
                    CachedAssetRow(VIEW_DOC, "img", 0, "a.png", 50L),
                    CachedAssetRow(VIEW_DOC, "img", 0, "b.png", 50L),
                )
            assertFails {
                val request = installRequest(s.revision(), generation = 2, assets = badAssets)
                s.store.installCachedDocument(s.session, request)
            }

            val persisted = checkNotNull(s.store.cachedDocument(s.scope, VIEW_DOC))
            assertEquals(100L, persisted.bytes)
            assertEquals(1L, persisted.generation)
            assertEquals(listOf("orig.html"), s.store.assetsForDocument(s.scope, VIEW_DOC).map { it.path })
        }

    @Test
    fun markDocumentSyncedNoOpsWithoutACachedRowAndUpdatesWhenPresent() =
        runTest {
            val store = store()
            val scope = "scope"

            store.markDocumentSynced(scope, "missing-doc", at = 10L)
            assertNull(store.cachedDocument(scope, "missing-doc"))

            store.upsertCachedDocument(scope, documentRow("doc_1"))
            store.markDocumentSynced(scope, "doc_1", at = 77L)
            assertEquals(77L, store.cachedDocument(scope, "doc_1")?.lastSyncedAt)
        }

    @Test
    fun removeCachedDocumentDeletesAllCachedRowsButLeavesOutbox() =
        runTest {
            val s = signedInStore()
            s.installNow(serverDocument(highlights = listOf(serverHighlight("hlt_1"))))
            s.enqueue(OutboxPayload.DocumentNote("note", null))

            s.store.removeCachedDocument(s.scope, VIEW_DOC)

            assertNull(s.store.cachedDocument(s.scope, VIEW_DOC))
            assertTrue(s.store.assetsForDocument(s.scope, VIEW_DOC).isEmpty())
            assertTrue(s.store.cachedHighlights(s.scope, VIEW_DOC).isEmpty())
            assertEquals(1, s.store.pendingOrdered(s.scope).size)
        }

    @Test
    fun catalogQueriesAreIsolatedByScope() =
        runTest {
            val store = store()
            store.upsertCachedDocument("scopeA", documentRow("doc_1"))

            assertNull(store.cachedDocument("scopeB", "doc_1"))
            assertTrue(store.cachedDocuments("scopeB").isEmpty())
            assertEquals(1, store.cachedDocuments("scopeA").size)
        }
}
