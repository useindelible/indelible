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

class DocumentEntitiesParityTest {
    private val jsonHeaders = headersOf(HttpHeaders.ContentType, "application/json")

    companion object {
        private const val FAR_FUTURE_EXPIRY = 4_102_444_800L
        private const val DOCUMENT_ID = "doc_01ABC"
        private const val EXPECTED_ITEM_COUNT = 3L
    }

    private fun entitiesJson() =
        """
        [
          {
            "object": "entity",
            "id": "ent_01",
            "name": "Sir Ken Robinson",
            "entity_type": "person",
            "item_count": $EXPECTED_ITEM_COUNT,
            "total_mentions": 5,
            "description": null,
            "created_at": "2026-01-01T00:00:00Z",
            "first_seen_at": "2026-01-01T00:00:00Z",
            "last_seen_at": "2026-01-01T00:00:00Z"
          },
          {
            "object": "entity",
            "id": "ent_02",
            "name": "TED",
            "entity_type": "organization",
            "item_count": 9,
            "total_mentions": 12,
            "description": null,
            "created_at": "2026-01-01T00:00:00Z",
            "first_seen_at": "2026-01-01T00:00:00Z",
            "last_seen_at": "2026-01-01T00:00:00Z"
          }
        ]
        """.trimIndent()

    private suspend fun newClient(engine: MockEngine): ApiClient {
        val tokenStorage = InMemoryTokenStorage()
        tokenStorage.saveToken("test-token")
        tokenStorage.saveExpiresAt(FAR_FUTURE_EXPIRY)
        return ApiClient(tokenStorage, engine = engine)
    }

    @Test
    fun listDocumentEntitiesGetsDocumentEntitiesPath() =
        runTest {
            var capturedMethod: HttpMethod? = null
            var capturedPath: String? = null
            var capturedAuth: String? = null
            val engine =
                MockEngine { request ->
                    capturedMethod = request.method
                    capturedPath = request.url.encodedPath
                    capturedAuth = request.headers[HttpHeaders.Authorization]
                    respond(entitiesJson(), HttpStatusCode.OK, jsonHeaders)
                }

            val result = newClient(engine).readerApiService.listDocumentEntities(DOCUMENT_ID)

            assertEquals(HttpMethod.Get, capturedMethod)
            assertEquals("/api/v1/documents/$DOCUMENT_ID/entities", capturedPath)
            assertEquals("Bearer test-token", capturedAuth)
            assertTrue(result.isSuccess)

            val entities = result.getOrThrow()
            assertEquals(2, entities.size)
            assertEquals("Sir Ken Robinson", entities.first().name)
            assertEquals("person", entities.first().entityType)
            assertEquals(EXPECTED_ITEM_COUNT, entities.first().itemCount)
            assertEquals("organization", entities[1].entityType)
        }

    @Test
    fun listDocumentEntitiesReturnsEmptyListWhenDocumentHasNone() =
        runTest {
            val engine = MockEngine { respond("[]", HttpStatusCode.OK, jsonHeaders) }

            val result = newClient(engine).readerApiService.listDocumentEntities(DOCUMENT_ID)

            assertTrue(result.isSuccess)
            assertTrue(result.getOrThrow().isEmpty())
        }
}
