package app.indelible.api

import app.indelible.api.generated.models.ArticleTocResponseStatus
import app.indelible.core.network.ApiClient
import app.indelible.core.storage.InMemoryTokenStorage
import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.respond
import io.ktor.http.HttpHeaders
import io.ktor.http.HttpMethod
import io.ktor.http.HttpStatusCode
import io.ktor.http.headersOf
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class ArticleTocParityTest {
    private val jsonHeaders = headersOf(HttpHeaders.ContentType, "application/json")

    companion object {
        private const val FAR_FUTURE_EXPIRY = 4_102_444_800L
        private const val DOCUMENT_ID = "doc_01ABC"
    }

    private fun readyJson() =
        """
        {
          "status": "ready",
          "truncated": false,
          "entries": [
            {
              "source_heading_index": 0,
              "id": "ind-toc-history",
              "title": "History",
              "depth": 0,
              "word_count": 312
            },
            {
              "source_heading_index": 2,
              "id": "ind-toc-the-ilari",
              "title": "The Ilari",
              "depth": 1,
              "word_count": 145
            }
          ]
        }
        """.trimIndent()

    private fun statusOnlyJson(status: String) =
        """{"status": "$status", "truncated": false, "entries": []}"""

    private suspend fun newClient(engine: MockEngine): ApiClient {
        val tokenStorage = InMemoryTokenStorage()
        tokenStorage.saveToken("test-token")
        tokenStorage.saveExpiresAt(FAR_FUTURE_EXPIRY)
        return ApiClient(tokenStorage, engine = engine)
    }

    @Test
    fun getArticleTocGetsDocumentTocPathWithAuth() =
        runTest {
            var capturedMethod: HttpMethod? = null
            var capturedPath: String? = null
            var capturedAuth: String? = null
            val engine =
                MockEngine { request ->
                    capturedMethod = request.method
                    capturedPath = request.url.encodedPath
                    capturedAuth = request.headers[HttpHeaders.Authorization]
                    respond(readyJson(), HttpStatusCode.OK, jsonHeaders)
                }

            val result = newClient(engine).readerApiService.getArticleToc(DOCUMENT_ID)

            assertEquals(HttpMethod.Get, capturedMethod)
            assertEquals("/api/v1/documents/$DOCUMENT_ID/toc", capturedPath)
            assertEquals("Bearer test-token", capturedAuth)
            assertTrue(result.isSuccess)

            val toc = result.getOrThrow()
            assertEquals(ArticleTocResponseStatus.READY, toc.status)
            assertEquals(2, toc.entries.size)
            assertEquals("ind-toc-history", toc.entries.first().id)
            assertEquals(0, toc.entries.first().sourceHeadingIndex)
            assertEquals(312, toc.entries.first().wordCount)
            assertEquals(1, toc.entries[1].depth)
        }

    @Test
    fun getArticleTocDeserializesPendingAndNone() =
        runTest {
            val cases =
                listOf(
                    "pending" to ArticleTocResponseStatus.PENDING,
                    "none" to ArticleTocResponseStatus.NONE,
                )
            for (status in cases) {
                val engine = MockEngine { respond(statusOnlyJson(status.first), HttpStatusCode.OK, jsonHeaders) }
                val result = newClient(engine).readerApiService.getArticleToc(DOCUMENT_ID)
                assertTrue(result.isSuccess)
                val toc = result.getOrThrow()
                assertEquals(status.second, toc.status)
                assertTrue(toc.entries.isEmpty())
            }
        }
}
