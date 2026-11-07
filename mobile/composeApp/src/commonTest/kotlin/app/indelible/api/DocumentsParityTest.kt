package app.indelible.api

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

class DocumentsParityTest {
    private val jsonHeaders = headersOf(HttpHeaders.ContentType, "application/json")

    companion object {
        private const val FAR_FUTURE_EXPIRY = 4_102_444_800L
        private const val DOCUMENT_ID = "doc_01ABC"
    }

    private fun noteJson() =
        """
        {"id": "note_01", "body": "My note", "created_at": "2026-01-01T00:00:00Z", "updated_at": "2026-01-01T00:00:00Z"}
        """.trimIndent()

    private fun documentAssetJson() =
        """
        {
            "id": "asset_01",
            "object": "document_asset",
            "asset_kind": "readable_html",
            "content_type": "text/html",
            "created_at": "2026-01-01T00:00:00Z",
            "document_id": "$DOCUMENT_ID",
            "download_url": "https://cdn.example.com/readable.html",
            "size_bytes": 1024,
            "status": "ready"
        }
        """.trimIndent()

    private suspend fun newClient(engine: MockEngine): ApiClient {
        val tokenStorage = InMemoryTokenStorage()
        tokenStorage.saveToken("test-token")
        tokenStorage.saveExpiresAt(FAR_FUTURE_EXPIRY)
        return ApiClient(tokenStorage, engine = engine)
    }

    @Test
    fun progressAndNotesUseDocumentRoutes() =
        runTest {
            val capturedPaths = mutableListOf<String>()
            val engine =
                MockEngine { request ->
                    capturedPaths += request.url.encodedPath
                    val body =
                        when (request.method) {
                            HttpMethod.Patch -> ""
                            else -> noteJson()
                        }
                    val status = if (request.method == HttpMethod.Patch) HttpStatusCode.NoContent else HttpStatusCode.OK
                    respond(body, status, jsonHeaders)
                }
            val apiClient = newClient(engine)

            apiClient.updateProgress(DOCUMENT_ID, 0.5f)
            val note = apiClient.getItemNote(DOCUMENT_ID)
            apiClient.upsertItemNote(DOCUMENT_ID, "My note")

            assertTrue(note.isSuccess)
            assertEquals("My note", note.getOrThrow()?.body)
            assertEquals(
                listOf(
                    "/api/v1/documents/$DOCUMENT_ID/progress",
                    "/api/v1/documents/$DOCUMENT_ID/note",
                    "/api/v1/documents/$DOCUMENT_ID/note",
                ),
                capturedPaths,
            )
        }

    @Test
    fun missingDocumentNoteMapsNotFoundToNull() =
        runTest {
            val engine =
                MockEngine {
                    respond(
                        """{"error":"not_found","message":"Document note not found"}""",
                        HttpStatusCode.NotFound,
                        jsonHeaders,
                    )
                }
            val apiClient = newClient(engine)

            val note = apiClient.getItemNote(DOCUMENT_ID)

            assertTrue(note.isSuccess)
            assertEquals(null, note.getOrThrow())
        }

    @Test
    fun assetsUseDocumentRoutes() =
        runTest {
            val capturedPaths = mutableListOf<String>()
            val engine =
                MockEngine { request ->
                    capturedPaths += request.url.encodedPath
                    respond(documentAssetJson(), HttpStatusCode.OK, jsonHeaders)
                }
            val apiClient = newClient(engine)

            val asset = apiClient.getAssetWithUrl(DOCUMENT_ID, "readable_html")

            assertEquals("https://cdn.example.com/readable.html", asset.getOrThrow().downloadUrl)
            assertEquals(
                listOf("/api/v1/documents/$DOCUMENT_ID/assets/readable_html"),
                capturedPaths,
            )
        }

    @Test
    fun streamAssetReadsBytesThroughDocumentProxy() =
        runTest {
            val capturedPaths = mutableListOf<String>()
            val html = "<html><body>readable</body></html>"
            val engine =
                MockEngine { request ->
                    capturedPaths += request.url.encodedPath
                    respond(html, HttpStatusCode.OK, headersOf(HttpHeaders.ContentType, "text/html"))
                }
            val apiClient = newClient(engine)

            val result = apiClient.streamAsset(DOCUMENT_ID, "readable_html")

            assertEquals(html, result.getOrThrow())
            // Must hit the reachable API proxy, never a presigned object-store URL.
            assertEquals(
                listOf("/api/v1/assets/documents/$DOCUMENT_ID/readable_html"),
                capturedPaths,
            )
        }

    @Test
    fun reprocessDocumentPostsToDocumentRoute() =
        runTest {
            var capturedMethod: HttpMethod? = null
            var capturedPath: String? = null
            var capturedAuthorization: String? = null
            val engine =
                MockEngine { request ->
                    capturedMethod = request.method
                    capturedPath = request.url.encodedPath
                    capturedAuthorization = request.headers[HttpHeaders.Authorization]
                    respond(
                        """{"queued":false,"job_type":"document.reprocess","retry_after_seconds":45}""",
                        HttpStatusCode.OK,
                        jsonHeaders,
                    )
                }
            val apiClient = newClient(engine)

            val result = apiClient.readerApiService.reprocessDocument(DOCUMENT_ID)

            assertEquals(HttpMethod.Post, capturedMethod)
            assertEquals("/api/v1/documents/$DOCUMENT_ID/reprocess", capturedPath)
            assertEquals("Bearer test-token", capturedAuthorization)
            assertTrue(result.isSuccess)
            assertEquals(false, result.getOrThrow().queued)
            assertEquals(45L, result.getOrThrow().retryAfterSeconds)
        }
}
