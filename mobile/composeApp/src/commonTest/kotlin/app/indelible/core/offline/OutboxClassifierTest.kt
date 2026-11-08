package app.indelible.core.offline

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs

private class FakeNetworkException : Exception("no network")

private class FakeLocalException : Exception("unparseable response")

private val isNetwork: (Throwable) -> Boolean = { it is FakeNetworkException }
private val NOTE = OutboxKind.DOCUMENT_NOTE
private val CREATE = OutboxKind.HIGHLIGHT_CREATE

private fun http(
    status: Int,
    body: String = "",
    problem: ApiProblem? = null,
) = SendOutcome.Http(status, retryAfterSeconds = null, message = body, problem = problem)

private fun failureOf(classification: Classification): SyncFailure = (classification as Classification.Terminal).failure

class OutboxClassifierTest {
    @Test
    fun row1NoNetworkIsRetryableWithFirstAttemptBackoff() {
        val classification =
            classify(NOTE, SendOutcome.Transport(FakeNetworkException()), attempts = 0, now = 1_000L, isNetwork)

        assertEquals(Classification.Retryable(1_000L + backoffMs(1)), classification)
    }

    @Test
    fun row3TimeoutAndHttp408AreRetryable() {
        val transportTimeout =
            classify(NOTE, SendOutcome.Transport(FakeNetworkException()), attempts = 1, now = 500L, isNetwork)
        val timeoutOutcome = SendOutcome.Http(408, retryAfterSeconds = null, message = "timeout")
        val http408 = classify(NOTE, timeoutOutcome, attempts = 1, now = 500L, isNetwork)

        assertEquals(Classification.Retryable(500L + backoffMs(2)), transportTimeout)
        assertEquals(Classification.Retryable(500L + backoffMs(2)), http408)
    }

    @Test
    fun row12ReplaySuccessIsDone() {
        val classification = classify(NOTE, SendOutcome.ReplaySuccess, attempts = 3, now = 0L, isNetwork)

        assertEquals(Classification.Done, classification)
    }

    @Test
    fun row13RejectedCarriesTheServerDetail() {
        val problem = ApiProblem(code = "validation_error", detail = "Note too long")
        val http400 = classify(NOTE, http(400, "bad request"), attempts = 0, now = 0L, isNetwork)
        val http422 = classify(NOTE, http(422, "{...}", problem), attempts = 0, now = 0L, isNetwork)

        val rejected = SyncFailure.Kind.REJECTED
        assertEquals(SyncFailure(rejected, status = 400, serverMessage = "bad request"), failureOf(http400))
        assertEquals(
            SyncFailure(rejected, status = 422, serverCode = "validation_error", serverMessage = "Note too long"),
            failureOf(http422),
        )
    }

    @Test
    fun row10NotFoundWithProblemBodyIsDeletedElsewhere() {
        val problem = ApiProblem(code = "not_found", detail = "document not found")
        val classification = classify(CREATE, http(404, "{...}", problem), attempts = 0, now = 0L, isNetwork)

        assertEquals(SyncFailure.Kind.DELETED_ELSEWHERE, failureOf(classification).kind)
        assertEquals("document not found", failureOf(classification).serverMessage)
    }

    @Test
    fun row11ALocatorValidationErrorOnACreateIsAnchorLost() {
        val problem =
            ApiProblem(code = "validation_error", errors = listOf(FieldError("locator.start_offset", "past the end")))
        val classification = classify(CREATE, http(422, "{...}", problem), attempts = 0, now = 0L, isNetwork)

        assertEquals(SyncFailure.Kind.ANCHOR_LOST, failureOf(classification).kind)
        assertEquals("past the end", failureOf(classification).serverMessage)
    }

    @Test
    fun row11aNonLocatorValidationErrorOnACreateIsRejected() {
        val problem = ApiProblem(code = "validation_error", errors = listOf(FieldError("color", "unknown colour")))
        val classification = classify(CREATE, http(422, "{...}", problem), attempts = 0, now = 0L, isNetwork)

        assertEquals(SyncFailure.Kind.REJECTED, failureOf(classification).kind)
    }

    @Test
    fun aCreateRejectedForUnpreparedContentKeepsRetryingAtTheHourlyCap() {
        val problem = ApiProblem(code = "validation_error", errors = listOf(FieldError("document_id", "not ready")))
        val create = classify(CREATE, http(422, "{...}", problem), attempts = 0, now = 1_000L, isNetwork)
        val note = classify(NOTE, http(422, "{...}", problem), attempts = 3, now = 1_000L, isNetwork)

        assertEquals(Classification.Retryable(1_000L + HOURLY_CAP_MS), create)
        assertEquals(Classification.Retryable(1_000L + HOURLY_CAP_MS), note)
    }

    @Test
    fun row17MethodNotAllowedIsServerTooOldForTheKindsFeature() {
        val events = classify(OutboxKind.READING_EVENT, http(405, "method not allowed"), 0, 0L, isNetwork)
        val colour = classify(OutboxKind.HIGHLIGHT_COLOR, http(405), 0, 0L, isNetwork)

        assertEquals(SyncFailure.Kind.SERVER_TOO_OLD, failureOf(events).kind)
        assertEquals(SyncFailure.Feature.READING_EVENTS, failureOf(events).feature)
        assertEquals(SyncFailure.Feature.HIGHLIGHTS, failureOf(colour).feature)
    }

    @Test
    fun row17BareNotFoundIsServerTooOld() {
        val classification = classify(NOTE, http(404), attempts = 0, now = 0L, isNetwork)

        assertEquals(SyncFailure.Kind.SERVER_TOO_OLD, failureOf(classification).kind)
        assertEquals(SyncFailure.Feature.NOTES, failureOf(classification).feature)
    }

    @Test
    fun aNonNetworkTransportFailureIsALocalError() {
        val classification = classify(NOTE, SendOutcome.Transport(FakeLocalException()), 0, 0L, isNetwork)

        assertEquals(SyncFailure.Kind.LOCAL_ERROR, failureOf(classification).kind)
        assertEquals("unparseable response", failureOf(classification).serverMessage)
    }

    @Test
    fun row14RateLimitedHonoursRetryAfter() {
        val rateLimited = SendOutcome.Http(429, retryAfterSeconds = 7, message = "slow down")
        val classification = classify(NOTE, rateLimited, attempts = 0, now = 1_000L, isNetwork)

        assertEquals(Classification.Retryable(1_000L + 7_000L), classification)
    }

    @Test
    fun retryAfterNeverParksARowBeyondTheHourlyCap() {
        val parked = SendOutcome.Http(503, retryAfterSeconds = 86_400, message = "later")
        val classification = classify(NOTE, parked, attempts = 0, now = 1_000L, isNetwork)

        assertEquals(Classification.Retryable(1_000L + HOURLY_CAP_MS), classification)
    }

    @Test
    fun retryAfterIsCappedBeforeItCanOverflow() {
        val absurd = SendOutcome.Http(503, retryAfterSeconds = Long.MAX_VALUE, message = "later")
        val classification = classify(NOTE, absurd, attempts = 0, now = 10_000L, isNetwork)

        assertEquals(Classification.Retryable(10_000L + HOURLY_CAP_MS), classification)
    }

    @Test
    fun row15ServerErrorIsRetryableWithBackoff() {
        val serverError = SendOutcome.Http(500, retryAfterSeconds = null, message = "boom")
        val classification = classify(NOTE, serverError, attempts = 2, now = 1_000L, isNetwork)

        assertEquals(Classification.Retryable(1_000L + backoffMs(3)), classification)
    }

    @Test
    fun row16StorageExceededUsesHourlyCapRegardlessOfAttemptsOrRetryAfter() {
        val withRetryAfter = SendOutcome.Http(507, retryAfterSeconds = 1, message = "quota")
        val withoutRetryAfter = SendOutcome.Http(507, retryAfterSeconds = null, message = "quota")
        val freshRow = classify(NOTE, withRetryAfter, attempts = 0, now = 1_000L, isNetwork)
        val manyAttempts = classify(NOTE, withoutRetryAfter, attempts = 40, now = 1_000L, isNetwork)

        assertEquals(Classification.Retryable(1_000L + backoffMs(6)), freshRow)
        assertEquals(freshRow, manyAttempts)
    }

    @Test
    fun http401And403AreAuthBlocked() {
        assertIs<Classification.AuthBlocked>(classify(NOTE, SendOutcome.Http(401, null, "expired"), 0, 0L, isNetwork))
        assertIs<Classification.AuthBlocked>(classify(NOTE, SendOutcome.Http(403, null, "forbidden"), 0, 0L, isNetwork))
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

    @Test
    fun row9ANoteConflictIsTerminalWithTheServersNote() {
        val conflict = SendOutcome.NoteConflict("theirs", 2_000L)
        val classification = classify(NOTE, conflict, attempts = 0, now = 0L, isNetwork)

        val expected =
            SyncFailure(SyncFailure.Kind.NOTE_CONFLICT, serverNoteBody = "theirs", serverNoteUpdatedAtEpochMs = 2_000L)
        assertEquals(expected, failureOf(classification))
    }
}
