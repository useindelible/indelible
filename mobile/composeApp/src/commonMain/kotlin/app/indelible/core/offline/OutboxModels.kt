package app.indelible.core.offline

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

enum class OutboxKind {
    READING_EVENT,
    HIGHLIGHT_CREATE,
    HIGHLIGHT_COLOR,
    HIGHLIGHT_NOTE,
    HIGHLIGHT_TAGS,
    HIGHLIGHT_DELETE,
    DOCUMENT_NOTE,
}

enum class OutboxState { PENDING, BLOCKED, FAILED }

@Serializable
sealed interface OutboxPayload {
    @Serializable
    @SerialName("reading_event")
    data class ReadingEvent(
        val eventId: String,
        val originSeq: Long,
        val kind: String,
        val progressBasisPoints: Int?,
        val cause: String?,
        val sessionId: String?,
        val attempt: Int?,
        val positionJson: String?,
        val assetKind: String?,
        val activeMs: Int?,
        val recordedAtEpochMs: Long,
    ) : OutboxPayload

    @Serializable
    @SerialName("highlight_create")
    data class HighlightCreate(
        val highlightId: String,
        val color: String,
        val textContent: String,
        val locatorJson: String?,
        val sourceLocatorJson: String?,
    ) : OutboxPayload

    @Serializable
    @SerialName("highlight_color")
    data class HighlightColor(
        val highlightId: String,
        val color: String,
    ) : OutboxPayload

    @Serializable
    @SerialName("highlight_note")
    data class HighlightNote(
        val highlightId: String,
        val body: String?,
    ) : OutboxPayload // null = delete note

    @Serializable
    @SerialName("highlight_tags")
    data class HighlightTags(
        val highlightId: String,
        val tags: List<String>,
    ) : OutboxPayload

    @Serializable
    @SerialName("highlight_delete")
    data class HighlightDelete(
        val highlightId: String,
    ) : OutboxPayload

    @Serializable
    @SerialName("document_note")
    data class DocumentNote(
        val body: String,
        val baseUpdatedAtEpochMs: Long?,
    ) : OutboxPayload
}

data class OutboxRow(
    val seq: Long,
    val scope: String,
    val id: String,
    val kind: OutboxKind,
    val entityId: String,
    val documentId: String,
    val payload: OutboxPayload,
    val createdAt: Long,
    val attempts: Int,
    val lastAttemptAt: Long?,
    val nextAttemptAt: Long,
    val state: OutboxState,
    val lastError: String?,
)

data class CachedDocumentRow(
    val documentId: String,
    val documentType: String,
    val title: String,
    val readerJson: String,
    val pinned: Boolean,
    val lastOpenedAt: Long,
    val lastSyncedAt: Long?,
    val bytes: Long,
)

data class CachedAssetRow(
    val documentId: String,
    val kind: String,
    val idx: Int,
    val path: String,
    val bytes: Long,
)

data class ClientIdentity(
    val clientId: String,
    val purgePending: Boolean,
)

data class CachedHighlightRow(
    val id: String,
    val documentId: String,
    val payloadJson: String,
    val updatedAt: Long,
)
