package app.indelible.offline.viewmodel

import app.indelible.core.offline.CatalogEntry
import app.indelible.core.offline.OutboxKind
import app.indelible.core.offline.STILL_PREPARING_ERROR
import app.indelible.core.offline.SyncFailure
import app.indelible.core.storage.DEFAULT_OFFLINE_CAP_BYTES

enum class ChangeStatus { PENDING, RETRYING, FAILED, BLOCKED, SUPERSEDED }

enum class StorageNotice { KEPT_MINE, KEPT_OTHER, CHANGED_REFRESHED }

private val RETRYABLE_KINDS = setOf(SyncFailure.Kind.REJECTED, SyncFailure.Kind.LOCAL_ERROR)
private val COPYABLE_KINDS = setOf(SyncFailure.Kind.DELETED_ELSEWHERE, SyncFailure.Kind.ANCHOR_LOST)

/** A queued change as the Storage screen lists it; [blocked] are the changes waiting on this failed create. */
data class PendingChange(
    val id: String,
    val kind: OutboxKind,
    val documentTitle: String?,
    val status: ChangeStatus,
    val error: String?,
    val createdAt: Long,
    val detail: String? = null,
    val blocked: List<PendingChange> = emptyList(),
    /** Its outbox rows in send order; a document's reading events are many rows but one change. */
    val rowIds: List<String> = listOf(id),
    val failure: SyncFailure? = null,
) {
    val canRetry: Boolean get() = status == ChangeStatus.FAILED && (failure == null || failure.kind in RETRYABLE_KINDS)

    val canDiscard: Boolean get() = canRetry || failure?.kind in COPYABLE_KINDS

    val copyText: String? get() = detail?.takeIf { failure?.kind in COPYABLE_KINDS }

    val stillPreparing: Boolean get() = status == ChangeStatus.RETRYING && error == STILL_PREPARING_ERROR
}

data class StorageUiState(
    val capBytes: Long = DEFAULT_OFFLINE_CAP_BYTES,
    val kept: List<CatalogEntry> = emptyList(),
    val cached: List<CatalogEntry> = emptyList(),
    val changes: List<PendingChange> = emptyList(),
    /** The phone is online but the server gave no answer; the queue waits rather than fails. */
    val serverUnreachable: Boolean = false,
) {
    val keptBytes: Long get() = kept.sumOf { it.bytes }

    val cachedBytes: Long get() = cached.sumOf { it.bytes }

    val usedBytes: Long get() = keptBytes + cachedBytes

    val itemCount: Int get() = kept.size + cached.size

    val keptOverCap: Boolean get() = keptBytes > capBytes

    val queuedChanges: Int
        get() = changes.flatMap { listOf(it) + it.blocked }.count { it.status != ChangeStatus.SUPERSEDED }
}
