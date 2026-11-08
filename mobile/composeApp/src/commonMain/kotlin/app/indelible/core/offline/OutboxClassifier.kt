package app.indelible.core.offline

sealed interface SendOutcome {
    /** [noteUpdatedAtEpochMs] is the server's stamp on a saved document note, so the copy can carry it. */
    data class Success(
        val noteUpdatedAtEpochMs: Long? = null,
    ) : SendOutcome

    /** 200 on a replay, or 404 on a delete that already applied. */
    data object ReplaySuccess : SendOutcome

    data class Http(
        val status: Int,
        val retryAfterSeconds: Long?,
        val message: String,
        val problem: ApiProblem? = null,
    ) : SendOutcome

    data class Transport(
        val cause: Throwable,
    ) : SendOutcome

    /** The server's note moved past the base this edit was made on; nothing was sent. */
    data class NoteConflict(
        val serverBody: String,
        val serverUpdatedAtEpochMs: Long,
    ) : SendOutcome

    /** The session changed before the request was dispatched; nothing reached the server. */
    data object Stale : SendOutcome
}

sealed interface Classification {
    data class Retryable(
        val nextAttemptAt: Long,
    ) : Classification

    data object AuthBlocked : Classification

    data class Terminal(
        val failure: SyncFailure,
    ) : Classification

    data object Done : Classification

    data object Stale : Classification
}

private const val BACKOFF_ATTEMPT_1_MS = 2_000L
private const val BACKOFF_ATTEMPT_2_MS = 8_000L
private const val BACKOFF_ATTEMPT_3_MS = 30_000L
private const val BACKOFF_ATTEMPT_4_MS = 120_000L
private const val BACKOFF_ATTEMPT_5_MS = 600_000L
internal const val HOURLY_CAP_MS = 3_600_000L
internal const val STILL_PREPARING_ERROR = "server_preparing_content"
private const val MILLIS_PER_SECOND = 1_000L

private const val HTTP_UNAUTHORIZED = 401
private const val HTTP_FORBIDDEN = 403
private const val HTTP_NOT_FOUND = 404
private const val HTTP_METHOD_NOT_ALLOWED = 405
private const val HTTP_TIMEOUT = 408
private const val HTTP_UNPROCESSABLE = 422
private const val HTTP_TOO_MANY_REQUESTS = 429
private const val HTTP_INSUFFICIENT_STORAGE = 507
private val SERVER_ERROR_RANGE = 500..599

private val BACKOFF_SCHEDULE_MS =
    longArrayOf(
        BACKOFF_ATTEMPT_1_MS,
        BACKOFF_ATTEMPT_2_MS,
        BACKOFF_ATTEMPT_3_MS,
        BACKOFF_ATTEMPT_4_MS,
        BACKOFF_ATTEMPT_5_MS,
    )

/** attempts=1 -> 2s, 2 -> 8s, 3 -> 30s, 4 -> 120s, 5 -> 600s, >=6 -> hourly, forever. */
fun backoffMs(attempts: Int): Long {
    val index = (attempts - 1).coerceAtLeast(0)
    return BACKOFF_SCHEDULE_MS.getOrElse(index) { HOURLY_CAP_MS }
}

fun classify(
    kind: OutboxKind,
    outcome: SendOutcome,
    attempts: Int,
    now: Long,
    isNetwork: (Throwable) -> Boolean,
): Classification =
    when (outcome) {
        is SendOutcome.Success, is SendOutcome.ReplaySuccess -> Classification.Done
        SendOutcome.Stale -> Classification.Stale
        is SendOutcome.NoteConflict ->
            Classification.Terminal(
                SyncFailure(
                    SyncFailure.Kind.NOTE_CONFLICT,
                    serverNoteBody = outcome.serverBody,
                    serverNoteUpdatedAtEpochMs = outcome.serverUpdatedAtEpochMs,
                ),
            )
        is SendOutcome.Transport ->
            if (isNetwork(outcome.cause)) {
                Classification.Retryable(now + backoffMs(attempts + 1))
            } else {
                val message = outcome.cause.message ?: outcome.cause.toString()
                Classification.Terminal(SyncFailure(SyncFailure.Kind.LOCAL_ERROR, serverMessage = message))
            }
        is SendOutcome.Http -> classifyHttp(kind, outcome, attempts, now)
    }

// 401/403 land here only post-refresh (the transport retries a pre-refresh 401 itself), so they
// mean the session, not this row, is unauthorized.
private fun classifyHttp(
    kind: OutboxKind,
    outcome: SendOutcome.Http,
    attempts: Int,
    now: Long,
): Classification {
    val status = outcome.status
    return when {
        status == HTTP_UNAUTHORIZED || status == HTTP_FORBIDDEN -> Classification.AuthBlocked
        status == HTTP_UNPROCESSABLE && outcome.unpreparedContent -> Classification.Retryable(now + HOURLY_CAP_MS)
        // 507 always waits the hourly cap regardless of Retry-After or attempt count: it signals
        // server-side capacity, not a per-row problem that shorter backoff would resolve sooner.
        status == HTTP_INSUFFICIENT_STORAGE -> Classification.Retryable(now + HOURLY_CAP_MS)
        status.isRetryable -> Classification.Retryable(now + retryAfterOrBackoff(outcome, attempts))
        else -> Classification.Terminal(terminalFailure(kind, outcome))
    }
}

private val Int.isRetryable: Boolean
    get() = this == HTTP_TIMEOUT || this == HTTP_TOO_MANY_REQUESTS || this in SERVER_ERROR_RANGE

// A server can ask for a long wait, but never for one longer than the schedule's own cap:
// a misconfigured header must not park a row for days.
private fun retryAfterOrBackoff(
    outcome: SendOutcome.Http,
    attempts: Int,
): Long =
    outcome.retryAfterSeconds
        ?.coerceIn(0, HOURLY_CAP_MS / MILLIS_PER_SECOND)
        ?.let { it * MILLIS_PER_SECOND }
        ?: backoffMs(attempts + 1)

private fun terminalFailure(
    kind: OutboxKind,
    outcome: SendOutcome.Http,
): SyncFailure =
    when (outcome.status) {
        HTTP_NOT_FOUND -> notFound(kind, outcome)
        HTTP_METHOD_NOT_ALLOWED -> serverTooOld(kind, outcome)
        else -> rejection(kind, outcome)
    }

// The server has no reader content for this item yet, so the change is kept until it has.
internal val SendOutcome.Http.unpreparedContent: Boolean
    get() = problem?.errors?.any { it.field == "document_id" } == true

private val SendOutcome.Http.locatorRejected: Boolean
    get() = problem?.errors?.any { it.field.startsWith("locator") || it.field.startsWith("source_locator") } == true

private fun SendOutcome.Http.failure(
    kind: SyncFailure.Kind,
    feature: SyncFailure.Feature? = null,
) = SyncFailure(
    kind = kind,
    status = status,
    serverCode = problem?.code,
    serverMessage = serverMessageOf(problem, message),
    feature = feature,
)

private fun rejection(
    kind: OutboxKind,
    outcome: SendOutcome.Http,
): SyncFailure =
    if (kind == OutboxKind.HIGHLIGHT_CREATE && outcome.status == HTTP_UNPROCESSABLE && outcome.locatorRejected) {
        outcome.failure(SyncFailure.Kind.ANCHOR_LOST)
    } else {
        outcome.failure(SyncFailure.Kind.REJECTED)
    }

// A bare 404 is a route the server does not have; a problem body is a document it no longer has.
private fun notFound(
    kind: OutboxKind,
    outcome: SendOutcome.Http,
): SyncFailure =
    when (outcome.problem?.code) {
        null -> serverTooOld(kind, outcome)
        "not_found" -> outcome.failure(SyncFailure.Kind.DELETED_ELSEWHERE)
        else -> outcome.failure(SyncFailure.Kind.REJECTED)
    }

private fun serverTooOld(
    kind: OutboxKind,
    outcome: SendOutcome.Http,
): SyncFailure = outcome.failure(SyncFailure.Kind.SERVER_TOO_OLD, kind.feature())
