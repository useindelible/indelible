package app.indelible.api

import app.indelible.core.network.ApiClient
import app.indelible.core.storage.InMemoryTokenStorage
import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.respond
import io.ktor.http.HttpMethod
import io.ktor.http.HttpStatusCode
import io.ktor.http.headersOf
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class ImportsParityTest {
    @Test
    fun rollbackImportSendsDeleteToRollback() =
        runTest {
            val tokenStorage = InMemoryTokenStorage()
            tokenStorage.saveToken("test-token")
            tokenStorage.saveExpiresAt(FAR_FUTURE_EXPIRY)
            var capturedMethod: HttpMethod? = null
            var capturedPath: String? = null
            val engine =
                MockEngine { request ->
                    capturedMethod = request.method
                    capturedPath = request.url.encodedPath
                    respond("", HttpStatusCode.NoContent, headersOf())
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val result = apiClient.rollbackImport("imp_123")

            assertEquals(HttpMethod.Delete, capturedMethod)
            assertEquals("/api/v1/imports/imp_123/rollback", capturedPath)
            assertTrue(result.isSuccess)
        }

    companion object {
        private const val FAR_FUTURE_EXPIRY = 4_102_444_800L
    }
}
