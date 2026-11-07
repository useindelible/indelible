package app.indelible.offline.viewmodel

import app.indelible.core.offline.Acquisition
import app.indelible.core.offline.CatalogEntry
import app.indelible.core.offline.DocumentSyncCounts

enum class Availability { KEPT, CACHED, DOWNLOADING, NOT_ON_DEVICE }

/** What a document shows about its queued changes; absent when none is waiting. */
sealed interface SyncBadge {
    data class Pending(
        val count: Int,
    ) : SyncBadge

    data object Retrying : SyncBadge

    data class Failed(
        val count: Int,
    ) : SyncBadge
}

data class DocumentOfflineStatus(
    val availability: Availability = Availability.NOT_ON_DEVICE,
    val sync: SyncBadge? = null,
    val bytes: Long? = null,
    val lastSyncedAt: Long? = null,
    val acquisition: Acquisition? = null,
)

fun documentOfflineStatus(
    copy: CatalogEntry?,
    acquisition: Acquisition?,
    counts: DocumentSyncCounts?,
): DocumentOfflineStatus =
    DocumentOfflineStatus(
        availability =
            when {
                copy?.pinned == true -> Availability.KEPT
                copy != null -> Availability.CACHED
                acquisition != null && acquisition !is Acquisition.Failed -> Availability.DOWNLOADING
                else -> Availability.NOT_ON_DEVICE
            },
        sync = counts?.let(::syncBadge),
        bytes = copy?.bytes,
        lastSyncedAt = copy?.lastSyncedAt,
        acquisition = acquisition,
    )

// A failure needs the user, so it outranks a retry, which outranks changes simply waiting.
private fun syncBadge(counts: DocumentSyncCounts): SyncBadge? {
    val stuck = counts.failed + counts.blocked
    return when {
        stuck > 0 -> SyncBadge.Failed(stuck)
        counts.retrying > 0 -> SyncBadge.Retrying
        counts.pending > 0 -> SyncBadge.Pending(counts.pending)
        else -> null
    }
}
