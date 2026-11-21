package app.indelible.core.offline

import kotlinx.serialization.Serializable

private const val MESSAGE_CUT = 200

/** Why a queued change can no longer be sent as it is; Storage reads the kind, never the text. */
@Serializable
data class SyncFailure(
    val kind: Kind,
    val status: Int? = null,
    val serverCode: String? = null,
    val serverMessage: String? = null,
    val feature: Feature? = null,
    val serverNoteBody: String? = null,
    val serverNoteUpdatedAtEpochMs: Long? = null,
) {
    enum class Kind { NOTE_CONFLICT, DELETED_ELSEWHERE, ANCHOR_LOST, REJECTED, SERVER_TOO_OLD, LOCAL_ERROR }

    enum class Feature { READING_EVENTS, HIGHLIGHTS, NOTES }
}

/** The RFC 7807 fields the phone reads from an error body. */
@Serializable
data class ApiProblem(
    val code: String? = null,
    val detail: String? = null,
    val errors: List<FieldError>? = null,
)

@Serializable
data class FieldError(
    val field: String,
    val message: String,
)

fun OutboxKind.feature(): SyncFailure.Feature =
    when (this) {
        OutboxKind.READING_EVENT -> SyncFailure.Feature.READING_EVENTS
        OutboxKind.DOCUMENT_NOTE -> SyncFailure.Feature.NOTES
        OutboxKind.HIGHLIGHT_CREATE,
        OutboxKind.HIGHLIGHT_COLOR,
        OutboxKind.HIGHLIGHT_NOTE,
        OutboxKind.HIGHLIGHT_TAGS,
        OutboxKind.HIGHLIGHT_DELETE,
        -> SyncFailure.Feature.HIGHLIGHTS
    }

fun parseProblem(body: String): ApiProblem? =
    runCatching { offlineJson.decodeFromString(ApiProblem.serializer(), body) }.getOrNull()

/** The server's own words for Storage: the detail, else the first field message, else the raw body. */
fun serverMessageOf(
    problem: ApiProblem?,
    body: String,
): String? =
    problem?.errors?.firstOrNull()?.message
        ?: problem?.detail
        ?: body.take(MESSAGE_CUT).takeIf { it.isNotBlank() }
