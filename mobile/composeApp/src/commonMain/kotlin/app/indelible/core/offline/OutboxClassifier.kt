package app.indelible.core.offline

sealed interface SendOutcome {
    data object Success : SendOutcome

    /** 200 on a replay, or 404 on a delete that already applied. */
    data object ReplaySuccess : SendOutcome

    data class Http(
        val status: Int,
        val retryAfterSeconds: Long?,
        val message: String,
    ) : SendOutcome

    data class Transport(
        val cause: Throwable,
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
        val error: String,
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
private const val MILLIS_PER_SECOND = 1_000L

private const val HTTP_BAD_REQUEST = 400
private const val HTTP_UNAUTHORIZED = 401
private const val HTTP_FORBIDDEN = 403
private const val HTTP_NOT_FOUND = 404
private const val HTTP_TIMEOUT = 408
private const val HTTP_CONFLICT = 409
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
    outcome: SendOutcome,
    attempts: Int,
    now: Long,
    isNetwork: (Throwable) -> Boolean,
): Classification =
    when (outcome) {
        is SendOutcome.Success, is SendOutcome.ReplaySuccess -> Classification.Done
        SendOutcome.Stale -> Classification.Stale
        is SendOutcome.Transport ->
            if (isNetwork(outcome.cause)) {
                Classification.Retryable(now + backoffMs(attempts + 1))
            } else {
                Classification.Terminal(outcome.cause.message ?: outcome.cause.toString())
            }
        is SendOutcome.Http -> classifyHttp(outcome, attempts, now)
    }

// 401/403 land here only post-refresh (the transport retries a pre-refresh 401 itself), so they
// mean the session, not this row, is unauthorized.
private fun classifyHttp(
    outcome: SendOutcome.Http,
    attempts: Int,
    now: Long,
): Classification {
    val status = outcome.status
    val isBadRequestOrConflict = status == HTTP_BAD_REQUEST || status == HTTP_CONFLICT
    val isNotFoundOrUnprocessable = status == HTTP_NOT_FOUND || status == HTTP_UNPROCESSABLE
    val isTerminalRejection = isBadRequestOrConflict || isNotFoundOrUnprocessable
    val isRetryableFailure = status == HTTP_TIMEOUT || status == HTTP_TOO_MANY_REQUESTS || status in SERVER_ERROR_RANGE
    return when {
        status == HTTP_UNAUTHORIZED || status == HTTP_FORBIDDEN -> Classification.AuthBlocked
        isTerminalRejection -> Classification.Terminal(outcome.message)
        // 507 always waits the hourly cap regardless of Retry-After or attempt count: it signals
        // server-side capacity, not a per-row problem that shorter backoff would resolve sooner.
        status == HTTP_INSUFFICIENT_STORAGE -> Classification.Retryable(now + HOURLY_CAP_MS)
        // A server can ask for a long wait, but never for one longer than the schedule's own cap:
        // a misconfigured header must not park a row for days.
        isRetryableFailure -> {
            val requested =
                outcome.retryAfterSeconds
                    ?.coerceIn(0, HOURLY_CAP_MS / MILLIS_PER_SECOND)
                    ?.let { it * MILLIS_PER_SECOND }
            Classification.Retryable(now + (requested ?: backoffMs(attempts + 1)))
        }
        else -> Classification.Terminal(outcome.message)
    }
}
