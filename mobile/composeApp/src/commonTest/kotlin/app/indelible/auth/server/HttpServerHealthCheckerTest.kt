package app.indelible.auth.server

import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.respond
import io.ktor.http.HttpMethod
import io.ktor.http.HttpStatusCode
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class HttpServerHealthCheckerTest {
    @Test
    fun healthyResponseSucceedsAgainstTheHealthEndpoint() =
        runTest {
            var capturedMethod: HttpMethod? = null
            var capturedUrl: String? = null
            val engine =
                MockEngine { request ->
                    capturedMethod = request.method
                    capturedUrl = request.url.toString()
                    respond("""{"status":"healthy"}""", HttpStatusCode.OK)
                }

            val result = HttpServerHealthChecker(engine).check("https://indelible.acme.dev")

            assertTrue(result.isSuccess)
            assertEquals(HttpMethod.Get, capturedMethod)
            assertEquals("https://indelible.acme.dev/api/health", capturedUrl)
        }

    @Test
    fun unhealthyStatusCodeFails() =
        runTest {
            val engine =
                MockEngine {
                    respond("""{"status":"unhealthy"}""", HttpStatusCode.ServiceUnavailable)
                }

            assertTrue(HttpServerHealthChecker(engine).check("https://indelible.acme.dev").isFailure)
        }

    @Test
    fun transportFailureFails() =
        runTest {
            val engine = MockEngine { throw IllegalStateException("connection refused") }

            assertTrue(HttpServerHealthChecker(engine).check("http://192.168.1.40:38473").isFailure)
        }

    @Test
    fun trailingSlashDoesNotDoubleUpInThePath() =
        runTest {
            var capturedUrl: String? = null
            val engine =
                MockEngine { request ->
                    capturedUrl = request.url.toString()
                    respond("""{"status":"healthy"}""", HttpStatusCode.OK)
                }

            HttpServerHealthChecker(engine).check("https://indelible.acme.dev/")

            assertEquals("https://indelible.acme.dev/api/health", capturedUrl)
        }
}
