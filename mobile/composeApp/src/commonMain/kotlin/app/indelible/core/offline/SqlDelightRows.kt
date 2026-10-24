package app.indelible.core.offline

import app.indelible.db.Cached_asset
import app.indelible.db.Cached_document
import app.indelible.db.Outbox
import kotlinx.serialization.decodeFromString

internal fun OutboxState.wireName(): String = name.lowercase()

internal fun Boolean.toLong(): Long = if (this) 1L else 0L

internal fun outboxRowFrom(row: Outbox): OutboxRow =
    OutboxRow(
        seq = row.seq,
        scope = row.scope,
        id = row.id,
        kind = OutboxKind.valueOf(row.kind.uppercase()),
        entityId = row.entity_id,
        documentId = row.document_id,
        payload = offlineJson.decodeFromString(row.payload_json),
        createdAt = row.created_at,
        attempts = row.attempts.toInt(),
        lastAttemptAt = row.last_attempt_at,
        nextAttemptAt = row.next_attempt_at,
        state = OutboxState.valueOf(row.state.uppercase()),
        lastError = row.last_error,
    )

internal fun cachedDocumentRowFrom(row: Cached_document): CachedDocumentRow =
    CachedDocumentRow(
        documentId = row.document_id,
        documentType = row.document_type,
        title = row.title,
        readerJson = row.reader_json,
        pinned = row.pinned != 0L,
        lastOpenedAt = row.last_opened_at,
        lastSyncedAt = row.last_synced_at,
        bytes = row.bytes,
    )

internal fun cachedAssetRowFrom(row: Cached_asset): CachedAssetRow =
    CachedAssetRow(
        documentId = row.document_id,
        kind = row.kind,
        idx = row.idx.toInt(),
        path = row.path,
        bytes = row.bytes,
    )
