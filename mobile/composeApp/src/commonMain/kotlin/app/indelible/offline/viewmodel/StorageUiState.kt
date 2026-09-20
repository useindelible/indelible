package app.indelible.offline.viewmodel

import app.indelible.core.offline.CatalogEntry
import app.indelible.core.offline.OutboxKind
import app.indelible.core.storage.DEFAULT_OFFLINE_CAP_BYTES

enum class ChangeStatus { PENDING, RETRYING, FAILED, BLOCKED, SUPERSEDED }

/** A queued change as the Storage screen lists it; [blocked] are the changes waiting on this failed create. */
data class PendingChange(
    val id: String,
    val kind: OutboxKind,
    val documentTitle: String?,
    val status: ChangeStatus,
    val error: String?,
    val blocked: List<PendingChange> = emptyList(),
) {
    val canRetry: Boolean get() = status == ChangeStatus.FAILED
}

data class StorageUiState(
    val capBytes: Long = DEFAULT_OFFLINE_CAP_BYTES,
    val kept: List<CatalogEntry> = emptyList(),
    val cached: List<CatalogEntry> = emptyList(),
    val changes: List<PendingChange> = emptyList(),
) {
    val usedBytes: Long get() = kept.sumOf { it.bytes } + cached.sumOf { it.bytes }

    /** Eviction only removes cached copies, so kept ones alone over the cap need the user. */
    val keptOverCap: Boolean get() = kept.sumOf { it.bytes } > capBytes

    val queuedChanges: Int
        get() = changes.flatMap { listOf(it) + it.blocked }.count { it.status != ChangeStatus.SUPERSEDED }
}
