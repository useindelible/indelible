package app.indelible.api

import app.indelible.core.network.ApiClient
import app.indelible.core.storage.InMemoryTokenStorage
import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.respond
import io.ktor.http.HttpHeaders
import io.ktor.http.HttpMethod
import io.ktor.http.HttpStatusCode
import io.ktor.http.content.TextContent
import io.ktor.http.headersOf
import kotlinx.coroutines.test.runTest
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.contentOrNull
import kotlinx.serialization.json.jsonArray
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNotNull
import kotlin.test.assertNull
import kotlin.test.assertTrue

class LibraryDocumentsParityTest {
    private val jsonHeaders = headersOf(HttpHeaders.ContentType, "application/json")

    companion object {
        private const val FAR_FUTURE_EXPIRY = 4_102_444_800L
        private const val LIBRARY_ENTRY_ID = "lib_01ABC"
        private const val DOCUMENT_ID = "doc_01ABC"
    }

    private fun libraryEntryJson() =
        """
        {
            "library_entry_id": "$LIBRARY_ENTRY_ID",
            "document_id": "$DOCUMENT_ID",
            "object": "library_entry",
            "title": "Test Article",
            "url": "https://example.com/article",
            "canonical_url": "https://example.com/article",
            "source": "manual",
            "document_type": "article",
            "triage_state": "inbox",
            "is_favorite": false,
            "is_shortlisted": false,
            "saved_at": "2026-03-25T12:00:00Z",
            "created_at": "2026-03-25T12:00:00Z",
            "updated_at": "2026-03-25T12:00:00Z"
        }
        """.trimIndent()

    private fun scopeCountsJson() =
        """
        {
            "total": 9,
            "unread": 4,
            "reading": 3,
            "done": 2,
            "by_item_type": [
                {"item_type": "article", "count": 7},
                {"item_type": "video", "count": 2}
            ]
        }
        """.trimIndent()

    private fun paginatedLibraryEntriesJson() =
        """
        {
            "data": [${libraryEntryJson()}],
            "page": {"has_more": false, "next_cursor": null}
        }
        """.trimIndent()

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
    fun listItemsQueriesLibraryWithoutFilterByDefault() =
        runTest {
            var capturedMethod: HttpMethod? = null
            var capturedPath: String? = null
            var capturedBody: String? = null
            val engine =
                MockEngine { request ->
                    capturedMethod = request.method
                    capturedPath = request.url.encodedPath
                    capturedBody = (request.body as? TextContent)?.text
                    respond(paginatedLibraryEntriesJson(), HttpStatusCode.OK, jsonHeaders)
                }

            val result = newClient(engine).listItems()

            assertEquals(HttpMethod.Post, capturedMethod)
            assertEquals("/api/v1/library/query", capturedPath)
            // No triage/type scope -> the unfiltered library (null filter expression).
            val body = Json.parseToJsonElement(assertNotNull(capturedBody)).jsonObject
            assertNull(body["filter_expression"]?.jsonPrimitive?.contentOrNull)
            assertTrue(result.isSuccess)
            assertEquals(
                LIBRARY_ENTRY_ID,
                result
                    .getOrThrow()
                    .data
                    .single()
                    .libraryEntryId,
            )
        }

    @Test
    fun listItemsComposesItemTypeAndTriageFilters() =
        runTest {
            var capturedBody: String? = null
            val engine =
                MockEngine { request ->
                    capturedBody = (request.body as? TextContent)?.text
                    respond(paginatedLibraryEntriesJson(), HttpStatusCode.OK, jsonHeaders)
                }

            newClient(engine).listItems(
                triageState = "inbox",
                itemType = "book",
                cursor = "after",
                limit = 25,
            )

            val body = Json.parseToJsonElement(assertNotNull(capturedBody)).jsonObject
            assertEquals("after", body["cursor"]?.jsonPrimitive?.content)
            assertEquals("25", body["limit"]?.jsonPrimitive?.content)

            val expression = assertNotNull(body["filter_expression"]).jsonObject
            assertEquals("and", expression["type"]?.jsonPrimitive?.content)
            val conditions = assertNotNull(expression["conditions"]).jsonArray
            val byField =
                conditions.associate { condition ->
                    val obj = condition.jsonObject
                    obj["field"]?.jsonPrimitive?.content to obj
                }
            assertEquals("book", byField["item_type"]?.get("value")?.jsonPrimitive?.content)
            assertEquals("eq", byField["item_type"]?.get("op")?.jsonPrimitive?.content)
            assertEquals("inbox", byField["triage_state"]?.get("value")?.jsonPrimitive?.content)
            assertEquals("eq", byField["triage_state"]?.get("op")?.jsonPrimitive?.content)
        }

    @Test
    fun getItemReadsLibraryEntryById() =
        runTest {
            var capturedMethod: HttpMethod? = null
            var capturedPath: String? = null
            val engine =
                MockEngine { request ->
                    capturedMethod = request.method
                    capturedPath = request.url.encodedPath
                    respond(libraryEntryJson(), HttpStatusCode.OK, jsonHeaders)
                }

            val result = newClient(engine).getItem(LIBRARY_ENTRY_ID)

            assertEquals(HttpMethod.Get, capturedMethod)
            assertEquals("/api/v1/library/$LIBRARY_ENTRY_ID", capturedPath)
            assertTrue(result.isSuccess)
            assertEquals(DOCUMENT_ID, result.getOrThrow().documentId)
        }

    @Test
    fun saveItemPostsToLibrary() =
        runTest {
            var capturedMethod: HttpMethod? = null
            var capturedPath: String? = null
            val engine =
                MockEngine { request ->
                    capturedMethod = request.method
                    capturedPath = request.url.encodedPath
                    respond(libraryEntryJson(), HttpStatusCode.Accepted, jsonHeaders)
                }

            val result =
                newClient(engine).saveItem(
                    app.indelible.core.model
                        .SaveItemRequest(url = "https://example.com/article"),
                )

            assertEquals(HttpMethod.Post, capturedMethod)
            assertEquals("/api/v1/library", capturedPath)
            assertTrue(result.isSuccess)
        }

    @Test
    fun deleteItemDeletesLibraryEntry() =
        runTest {
            var capturedMethod: HttpMethod? = null
            var capturedPath: String? = null
            val engine =
                MockEngine { request ->
                    capturedMethod = request.method
                    capturedPath = request.url.encodedPath
                    respond("", HttpStatusCode.NoContent, jsonHeaders)
                }

            val result = newClient(engine).deleteItem(LIBRARY_ENTRY_ID)

            assertEquals(HttpMethod.Delete, capturedMethod)
            assertEquals("/api/v1/library/$LIBRARY_ENTRY_ID", capturedPath)
            assertTrue(result.isSuccess)
        }

    @Test
    fun triageAndTogglesUseLibraryEntryRoutes() =
        runTest {
            val capturedPaths = mutableListOf<String>()
            val engine =
                MockEngine { request ->
                    capturedPaths += request.url.encodedPath
                    respond(libraryEntryJson(), HttpStatusCode.OK, jsonHeaders)
                }
            val apiClient = newClient(engine)

            apiClient.triageItem(LIBRARY_ENTRY_ID, "archive")
            apiClient.toggleFavorite(LIBRARY_ENTRY_ID)
            apiClient.toggleShortlist(LIBRARY_ENTRY_ID)
            apiClient.restoreItem(LIBRARY_ENTRY_ID)

            assertEquals(
                listOf(
                    "/api/v1/library/$LIBRARY_ENTRY_ID/triage",
                    "/api/v1/library/$LIBRARY_ENTRY_ID/favorite",
                    "/api/v1/library/$LIBRARY_ENTRY_ID/shortlist",
                    "/api/v1/library/$LIBRARY_ENTRY_ID/restore",
                ),
                capturedPaths,
            )
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
    fun tagsUseLibraryEntryRoutes() =
        runTest {
            val capturedPaths = mutableListOf<String>()
            val engine =
                MockEngine { request ->
                    capturedPaths += request.url.encodedPath
                    respond("""{"tags":["kotlin"]}""", HttpStatusCode.OK, jsonHeaders)
                }
            val apiClient = newClient(engine)

            val current = apiClient.getItemTags(LIBRARY_ENTRY_ID)
            val updated = apiClient.setItemTags(LIBRARY_ENTRY_ID, listOf("kotlin"))

            assertEquals(listOf("kotlin"), current.getOrThrow())
            assertEquals(listOf("kotlin"), updated.getOrThrow())
            assertEquals(
                listOf(
                    "/api/v1/library/$LIBRARY_ENTRY_ID/tags",
                    "/api/v1/library/$LIBRARY_ENTRY_ID/tags",
                ),
                capturedPaths,
            )
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

    @Test
    fun authFailureIsReported() =
        runTest {
            val tokenStorage = InMemoryTokenStorage()
            tokenStorage.saveToken("test-token")
            tokenStorage.saveExpiresAt(FAR_FUTURE_EXPIRY)
            val engine =
                MockEngine {
                    respond("""{"error":"unauthorized"}""", HttpStatusCode.Unauthorized, jsonHeaders)
                }

            val result = ApiClient(tokenStorage, engine = engine).getItem(LIBRARY_ENTRY_ID)

            assertTrue(result.isFailure)
            assertNotNull(result.exceptionOrNull())
        }

    @Test
    fun getScopeCountsReadsCountsWithoutTriageByDefault() =
        runTest {
            var capturedMethod: HttpMethod? = null
            var capturedPath: String? = null
            var capturedQuery: String? = null
            var capturedAuth: String? = null
            val engine =
                MockEngine { request ->
                    capturedMethod = request.method
                    capturedPath = request.url.encodedPath
                    capturedQuery = request.url.encodedQuery
                    capturedAuth = request.headers[HttpHeaders.Authorization]
                    respond(scopeCountsJson(), HttpStatusCode.OK, jsonHeaders)
                }

            val result = newClient(engine).libraryApiService.getScopeCounts()

            assertEquals(HttpMethod.Get, capturedMethod)
            assertEquals("/api/v1/library/counts", capturedPath)
            assertEquals("", capturedQuery)
            assertEquals("Bearer test-token", capturedAuth)
            assertTrue(result.isSuccess)

            val counts = result.getOrThrow()
            assertEquals(9L, counts.total)
            assertEquals(4L, counts.unread)
            assertEquals(3L, counts.reading)
            assertEquals(2L, counts.done)
            assertEquals(2, counts.byItemType.size)
            assertEquals("article", counts.byItemType.first().itemType)
            assertEquals(7L, counts.byItemType.first().count)
        }

    @Test
    fun getScopeCountsPassesTriageStateQuery() =
        runTest {
            var capturedQuery: String? = null
            val engine =
                MockEngine { request ->
                    capturedQuery = request.url.encodedQuery
                    respond(scopeCountsJson(), HttpStatusCode.OK, jsonHeaders)
                }

            val result = newClient(engine).libraryApiService.getScopeCounts(triageState = "later")

            assertEquals("triage_state=later", capturedQuery)
            assertTrue(result.isSuccess)
        }
}
