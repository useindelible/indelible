package app.indelible.offline.viewmodel

import app.indelible.core.offline.Acquisition
import app.indelible.core.offline.CatalogEntry
import app.indelible.core.offline.DocumentSyncCounts
import app.indelible.core.offline.DownloadFailure
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull

class DocumentOfflineStatusTest {
    private fun copy(pinned: Boolean) =
        CatalogEntry(
            documentId = "doc_1",
            documentType = "article",
            title = "Title",
            pinned = pinned,
            lastOpenedAt = 1L,
            lastSyncedAt = 7L,
            bytes = 42L,
            generation = 1L,
        )

    private fun counts(
        pending: Int = 0,
        retrying: Int = 0,
        failed: Int = 0,
        blocked: Int = 0,
    ) = DocumentSyncCounts(pending, retrying, failed, blocked)

    private fun availability(
        copy: CatalogEntry?,
        acquisition: Acquisition?,
    ) = documentOfflineStatus(copy, acquisition, null).availability

    private fun sync(counts: DocumentSyncCounts?) = documentOfflineStatus(null, null, counts).sync

    @Test
    fun keptWhenPinnedCachedWhenPresent() {
        assertEquals(Availability.KEPT, availability(copy(pinned = true), null))
        assertEquals(Availability.CACHED, availability(copy(pinned = false), null))
        assertEquals(Availability.DOWNLOADING, availability(null, Acquisition.Downloading(0.5f)))
        assertEquals(Availability.DOWNLOADING, availability(null, Acquisition.WaitingForConnection))
        assertEquals(Availability.NOT_ON_DEVICE, availability(null, Acquisition.Failed(DownloadFailure.NO_SPACE)))
        assertEquals(Availability.NOT_ON_DEVICE, availability(null, null))
        val kept = documentOfflineStatus(copy(pinned = true), null, null)
        assertEquals(42L, kept.bytes)
        assertEquals(7L, kept.lastSyncedAt)
    }

    @Test
    fun noSyncBadgeWithoutRows() {
        assertNull(sync(null))
        assertNull(sync(counts()))
    }

    @Test
    fun retryingIsPendingWithAttempts() {
        assertEquals(SyncBadge.Pending(2), sync(counts(pending = 2)))
        assertEquals(SyncBadge.Retrying, sync(counts(pending = 2, retrying = 1)))
    }

    @Test
    fun failedAndBlockedCounted() {
        assertEquals(SyncBadge.Failed(3), sync(counts(pending = 1, retrying = 1, failed = 1, blocked = 2)))
    }
}
