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
            val store = store()
            val scope = "scope"

            store.installCachedDocument(
                scope,
                documentRow("doc_1", bytes = 10L),
                listOf(CachedAssetRow("doc_1", "html", 0, "v1.html", 10L)),
            )
            store.installCachedDocument(
                scope,
                documentRow("doc_1", bytes = 20L),
                listOf(
                    CachedAssetRow("doc_1", "html", 0, "v2.html", 15L),
                    CachedAssetRow("doc_1", "img", 0, "v2.png", 5L),
                ),
            )

            assertEquals(20L, store.cachedDocument(scope, "doc_1")?.bytes)
            val assets = store.assetsForDocument(scope, "doc_1")
            assertEquals(setOf("v2.html", "v2.png"), assets.map { it.path }.toSet())
        }

    @Test
    fun installCachedDocumentFailureMidListLeavesPreviousInstallIntact() =
        runTest {
            val store = store()
            val scope = "scope"
            store.installCachedDocument(
                scope,
                documentRow("doc_1", bytes = 100L),
                listOf(CachedAssetRow("doc_1", "html", 0, "orig.html", 100L)),
            )

            val badAssets =
                listOf(
                    CachedAssetRow("doc_1", "img", 0, "a.png", 50L),
                    CachedAssetRow("doc_1", "img", 0, "b.png", 50L),
                )
            assertFails {
                store.installCachedDocument(scope, documentRow("doc_1", bytes = 200L), badAssets)
            }

            val persisted = store.cachedDocument(scope, "doc_1")!!
            assertEquals(100L, persisted.bytes)
            val assets = store.assetsForDocument(scope, "doc_1")
            assertEquals(listOf("orig.html"), assets.map { it.path })
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
            val store = store()
            val scope = "scope"
            store.installCachedDocument(
                scope,
                documentRow("doc_1"),
                listOf(CachedAssetRow("doc_1", "html", 0, "v1.html", 10L)),
            )
            store.upsertCachedHighlight(scope, "hlt_1", "doc_1", "{}", 1L)
            store.enqueue(scope, OutboxKind.DOCUMENT_NOTE, "doc_1", "doc_1") {
                OutboxPayload.DocumentNote("note", null) to Unit
            }

            store.removeCachedDocument(scope, "doc_1")

            assertNull(store.cachedDocument(scope, "doc_1"))
            assertTrue(store.assetsForDocument(scope, "doc_1").isEmpty())
            assertEquals(1, store.drainable(scope, Long.MAX_VALUE).size)
            val remainingHighlight =
                store.enqueue(scope, OutboxKind.HIGHLIGHT_DELETE, "hlt_1", "doc_1") {
                    val existing = getCachedHighlight("hlt_1")
                    OutboxPayload.HighlightDelete("hlt_1") to existing
                }
            assertNull(remainingHighlight)
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
