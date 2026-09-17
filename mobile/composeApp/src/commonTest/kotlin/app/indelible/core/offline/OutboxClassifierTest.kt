package app.indelible.core.offline

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs

private class FakeNetworkException : Exception("no network")

private class FakeLocalException : Exception("unparseable response")

private val isNetwork: (Throwable) -> Boolean = { it is FakeNetworkException }

class OutboxClassifierTest {
    @Test
    fun row1NoNetworkIsRetryableWithFirstAttemptBackoff() {
        val classification =
            classify(SendOutcome.Transport(FakeNetworkException()), attempts = 0, now = 1_000L, isNetwork)

        assertEquals(Classification.Retryable(1_000L + backoffMs(1)), classification)
    }

    @Test
    fun row3TimeoutAndHttp408AreRetryable() {
        val transportTimeout =
            classify(SendOutcome.Transport(FakeNetworkException()), attempts = 1, now = 500L, isNetwork)
        val timeoutOutcome = SendOutcome.Http(408, retryAfterSeconds = null, message = "timeout")
        val http408 = classify(timeoutOutcome, attempts = 1, now = 500L, isNetwork)

        assertEquals(Classification.Retryable(500L + backoffMs(2)), transportTimeout)
        assertEquals(Classification.Retryable(500L + backoffMs(2)), http408)
    }

    @Test
    fun row12ReplaySuccessIsDone() {
        val classification = classify(SendOutcome.ReplaySuccess, attempts = 3, now = 0L, isNetwork)

        assertEquals(Classification.Done, classification)
    }

    @Test
    fun row13ValidationRejectionIsTerminal() {
        val http400 = classify(SendOutcome.Http(400, null, "bad request"), attempts = 0, now = 0L, isNetwork)
        val http422 = classify(SendOutcome.Http(422, null, "unprocessable"), attempts = 0, now = 0L, isNetwork)

        assertEquals(Classification.Terminal("bad request"), http400)
        assertEquals(Classification.Terminal("unprocessable"), http422)
    }

    @Test
    fun row14RateLimitedHonoursRetryAfter() {
        val rateLimited = SendOutcome.Http(429, retryAfterSeconds = 7, message = "slow down")
        val classification = classify(rateLimited, attempts = 0, now = 1_000L, isNetwork)

        assertEquals(Classification.Retryable(1_000L + 7_000L), classification)
    }

    @Test
    fun row15ServerErrorIsRetryableWithBackoff() {
        val serverError = SendOutcome.Http(500, retryAfterSeconds = null, message = "boom")
        val classification = classify(serverError, attempts = 2, now = 1_000L, isNetwork)

        assertEquals(Classification.Retryable(1_000L + backoffMs(3)), classification)
    }

    @Test
    fun row16StorageExceededUsesHourlyCapRegardlessOfAttemptsOrRetryAfter() {
        val withRetryAfter = SendOutcome.Http(507, retryAfterSeconds = 1, message = "quota")
        val withoutRetryAfter = SendOutcome.Http(507, retryAfterSeconds = null, message = "quota")
        val freshRow = classify(withRetryAfter, attempts = 0, now = 1_000L, isNetwork)
        val manyAttempts = classify(withoutRetryAfter, attempts = 40, now = 1_000L, isNetwork)

        assertEquals(Classification.Retryable(1_000L + backoffMs(6)), freshRow)
        assertEquals(freshRow, manyAttempts)
    }

    @Test
    fun http401And403AreAuthBlocked() {
        assertIs<Classification.AuthBlocked>(classify(SendOutcome.Http(401, null, "expired"), 0, 0L, isNetwork))
        assertIs<Classification.AuthBlocked>(classify(SendOutcome.Http(403, null, "forbidden"), 0, 0L, isNetwork))
    }

    @Test
    fun unmappedHttpStatusIsTerminal() {
        assertEquals(
            Classification.Terminal("method not allowed"),
            classify(SendOutcome.Http(405, null, "method not allowed"), attempts = 0, now = 0L, isNetwork),
        )
    }

    @Test
    fun unclassifiableLocalExceptionSurfacesAsTerminal() {
        val classification = classify(SendOutcome.Transport(FakeLocalException()), attempts = 0, now = 0L, isNetwork)

        assertEquals(Classification.Terminal("unparseable response"), classification)
    }

    @Test
    fun backoffScheduleMatchesTheSpecifiedTable() {
        assertEquals(2_000L, backoffMs(1))
        assertEquals(8_000L, backoffMs(2))
        assertEquals(30_000L, backoffMs(3))
        assertEquals(120_000L, backoffMs(4))
        assertEquals(600_000L, backoffMs(5))
        assertEquals(3_600_000L, backoffMs(6))
    }

    @Test
    fun backoffNeverExcludesARowEvenAtHighAttemptCounts() {
        assertEquals(3_600_000L, backoffMs(40))
    }
}
