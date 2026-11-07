package app.indelible.core.network

import app.indelible.core.storage.InMemoryTokenStorage
import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.respond
import io.ktor.client.request.get
import io.ktor.http.HttpStatusCode
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.test.runTest
import kotlinx.io.IOException
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull

class ReachabilityTest {
    private var reply: () -> HttpStatusCode = { HttpStatusCode.OK }
    private var failure: Throwable? = null

    private val transport =
        AuthenticatedApiTransport(
            tokenStorage = InMemoryTokenStorage(),
            engine =
                MockEngine {
                    failure?.let { throw it }
                    respond("", reply())
                },
        )

    private suspend fun send() {
        runCatching { transport.httpClient.get("http://server.test/api/v1/ping") }
    }

    @Test
    fun unknownUntilARequestEnds() {
        assertNull(transport.reachable.value)
    }

    @Test
    fun anAnswerFromTheServerMeansReachable() =
        runTest {
            for (status in listOf(HttpStatusCode.OK, HttpStatusCode.NotFound, HttpStatusCode.InternalServerError)) {
                reply = { status }
                failure = IOException("down")
                send()
                failure = null
                send()

                assertEquals(true, transport.reachable.value, "$status")
            }
        }

    @Test
    fun noAnswerOrAGatewayWithoutItsServerMeansUnreachable() =
        runTest {
            failure = IOException("no route")
            send()
            assertEquals(false, transport.reachable.value)

            failure = null
            val gatewayErrors =
                listOf(HttpStatusCode.BadGateway, HttpStatusCode.ServiceUnavailable, HttpStatusCode.GatewayTimeout)
            for (status in gatewayErrors) {
                reply = { HttpStatusCode.OK }
                send()
                reply = { status }
                send()

                assertEquals(false, transport.reachable.value, "$status")
            }
        }

    @Test
    fun aCancelledRequestTellsNothing() =
        runTest {
            send()
            failure = CancellationException("left the screen")
            send()

            assertEquals(true, transport.reachable.value)
        }
}
