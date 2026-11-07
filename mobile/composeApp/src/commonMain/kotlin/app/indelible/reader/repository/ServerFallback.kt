package app.indelible.reader.repository

import app.indelible.core.network.ApiException
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.withTimeoutOrNull
import okio.IOException

private const val CLIENT_ERROR_MIN = 400
private const val CLIENT_ERROR_MAX = 499
private const val REQUEST_TIMEOUT = 408
private const val TOO_MANY_REQUESTS = 429

/** [fetch]'s result, or an [IOException] when the server gives no answer within [budgetMs]. */
internal suspend fun <S> attemptWithin(
    budgetMs: Long,
    fetch: suspend () -> Result<S>,
): Result<S> {
    val fetched =
        withTimeoutOrNull(budgetMs) { fetch() }
            ?: Result.failure(IOException("No answer in $budgetMs ms"))
    fetched.exceptionOrNull()?.let { if (it is CancellationException) throw it }
    return fetched
}

// A 4xx is the server's real answer, which a kept copy must not hide; 408 and 429 mean it just can't answer now.
internal fun Throwable.fallsBack(): Boolean =
    this !is ApiException ||
        statusCode !in CLIENT_ERROR_MIN..CLIENT_ERROR_MAX ||
        statusCode == REQUEST_TIMEOUT ||
        statusCode == TOO_MANY_REQUESTS
