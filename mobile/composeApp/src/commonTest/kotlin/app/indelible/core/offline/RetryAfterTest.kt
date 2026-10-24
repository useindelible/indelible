package app.indelible.core.offline

import app.indelible.core.network.AuthenticatedApiTransport
import app.indelible.core.storage.InMemoryTokenStorage
import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.respond
import io.ktor.http.HttpHeaders
import io.ktor.http.HttpStatusCode
import io.ktor.http.headersOf
import io.ktor.http.toHttpDate
import io.ktor.util.date.GMTDate
import io.ktor.util.date.getTimeMillis
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlin.test.assertNull

private const val SCOPE = "http://localhost:38473|usr_1"
private val SESSION = Session(epoch = 0, origin = "http://localhost:38473", scope = SCOPE)
private const val NINETY_SECONDS_MS = 90_000L
private const val FAR_FUTURE_EXPIRY = 4_102_444_800L

class RetryAfterTest {
    private suspend fun retryAfterFor(
        status: HttpStatusCode,
        header: String,
    ): SendOutcome.Http {
        val tokenStorage = InMemoryTokenStorage()
        tokenStorage.saveToken("test-token")
        tokenStorage.saveExpiresAt(FAR_FUTURE_EXPIRY)
        val registry = SessionRegistry().apply { publish(SessionState(0, SESSION)) }
        val engine = MockEngine { respond("slow down", status, headersOf(HttpHeaders.RetryAfter, header)) }
        val sender = ApiOutboxSender(AuthenticatedApiTransport(tokenStorage, engine = engine, registry = registry))
        val row =
            OutboxRow(
                seq = 1,
                scope = SCOPE,
                id = "obx_1",
                kind = OutboxKind.DOCUMENT_NOTE,
                entityId = "doc_01",
                documentId = "doc_01",
                payload = OutboxPayload.DocumentNote("note body", null),
                createdAt = 0,
                attempts = 0,
                lastAttemptAt = null,
                nextAttemptAt = 0,
                state = OutboxState.PENDING,
                lastError = null,
            )
        return assertIs<SendOutcome.Http>(sender.send(SESSION, "cli_test", listOf(row)))
    }

    @Test
    fun deltaSecondsIsUsedAsIs() =
        runTest {
            assertEquals(SendOutcome.Http(429, 120L, "slow down"), retryAfterFor(HttpStatusCode.TooManyRequests, "120"))
        }

    @Test
    fun httpDateAheadOfNowBecomesTheRemainingSeconds() =
        runTest {
            val header = GMTDate(getTimeMillis() + NINETY_SECONDS_MS).toHttpDate()
            val outcome = retryAfterFor(HttpStatusCode.ServiceUnavailable, header)
            assertEquals(503, outcome.status)
            val seconds = outcome.retryAfterSeconds
            assertEquals(true, seconds != null && seconds in 88L..92L, "retryAfterSeconds=$seconds")
        }

    @Test
    fun httpDateInThePastClampsToZero() =
        runTest {
            val outcome = retryAfterFor(HttpStatusCode.ServiceUnavailable, "Wed, 21 Oct 2015 07:28:00 GMT")
            assertEquals(0L, outcome.retryAfterSeconds)
        }

    @Test
    fun negativeDeltaSecondsYieldsNoDelay() =
        runTest {
            assertNull(retryAfterFor(HttpStatusCode.TooManyRequests, "-5").retryAfterSeconds)
        }

    @Test
    fun unparsableValueYieldsNoDelay() =
        runTest {
            assertNull(retryAfterFor(HttpStatusCode.TooManyRequests, "soon").retryAfterSeconds)
        }
}
