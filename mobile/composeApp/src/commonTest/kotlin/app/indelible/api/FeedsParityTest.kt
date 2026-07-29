package app.indelible.api

import app.indelible.core.network.ApiClient
import app.indelible.core.storage.InMemoryTokenStorage
import app.indelible.feed.model.UpdateSubscriptionRequest
import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.respond
import io.ktor.http.HttpHeaders
import io.ktor.http.HttpMethod
import io.ktor.http.HttpStatusCode
import io.ktor.http.headersOf
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNotNull
import kotlin.test.assertTrue

class FeedsParityTest {
    private val jsonHeaders = headersOf(HttpHeaders.ContentType, "application/json")

    @Test
    fun listFeedItemsSendsGetToFeedDeliveries() =
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
                    respond(paginatedFeedItemsJson(), HttpStatusCode.OK, jsonHeaders)
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val result = apiClient.listFeedItems()

            assertEquals(HttpMethod.Get, capturedMethod)
            assertEquals("/api/v1/feeds/deliveries", capturedPath)
            assertTrue(result.isSuccess)
        }

    @Test
    fun listFeedItemsPassesStateParam() =
        runTest {
            val tokenStorage = InMemoryTokenStorage()
            tokenStorage.saveToken("test-token")
            tokenStorage.saveExpiresAt(FAR_FUTURE_EXPIRY)
            var capturedQuery: String? = null
            val engine =
                MockEngine { request ->
                    capturedQuery = request.url.parameters["state"]
                    respond(paginatedFeedItemsJson(), HttpStatusCode.OK, jsonHeaders)
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            apiClient.listFeedItems(state = "unread")

            assertEquals("unread", capturedQuery)
        }

    @Test
    fun getFeedItemSendsGetWithId() =
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
                    respond(feedItemWithStateJson(), HttpStatusCode.OK, jsonHeaders)
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val result = apiClient.getFeedItem("fd_01ABC")

            assertEquals(HttpMethod.Get, capturedMethod)
            assertEquals("/api/v1/feeds/deliveries/fd_01ABC", capturedPath)
            assertTrue(result.isSuccess)
        }

    @Test
    fun markFeedItemSeenSendsPost() =
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
                    respond(feedItemWithStateJson(), HttpStatusCode.OK, jsonHeaders)
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val result = apiClient.markFeedItemSeen("fd_01ABC")

            assertEquals(HttpMethod.Post, capturedMethod)
            assertEquals("/api/v1/feeds/deliveries/fd_01ABC/seen", capturedPath)
            assertTrue(result.isSuccess)
        }

    @Test
    fun saveFeedItemToLibrarySendsPost() =
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
                    respond(libraryEntryJson(), HttpStatusCode.OK, jsonHeaders)
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val result = apiClient.saveFeedItemToLibrary("fd_01ABC")

            assertEquals(HttpMethod.Post, capturedMethod)
            assertEquals("/api/v1/library/from-delivery", capturedPath)
            assertTrue(result.isSuccess)
        }

    @Test
    fun prepareFeedDeliverySendsPostAndReturnsDocumentId() =
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
                    respond("""{"document_id":"doc_99"}""", HttpStatusCode.OK, jsonHeaders)
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val result = apiClient.prepareFeedDelivery("fd_01ABC")

            assertEquals(HttpMethod.Post, capturedMethod)
            assertEquals("/api/v1/feeds/deliveries/fd_01ABC/prepare", capturedPath)
            assertEquals("doc_99", result.getOrThrow().documentId)
        }

    @Test
    fun markAllFeedItemsSeenSendsPost() =
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
                    respond("""{"updated":0}""", HttpStatusCode.OK, jsonHeaders)
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val result = apiClient.markAllFeedItemsSeen()

            assertEquals(HttpMethod.Post, capturedMethod)
            assertEquals("/api/v1/feeds/deliveries/mark-all-seen", capturedPath)
            assertTrue(result.isSuccess)
        }

    @Test
    fun listFeedSubscriptionsSendsGet() =
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
                    respond(paginatedSubscriptionsJson(), HttpStatusCode.OK, jsonHeaders)
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val result = apiClient.listFeedSubscriptions()

            assertEquals(HttpMethod.Get, capturedMethod)
            assertEquals("/api/v1/feeds/subscriptions", capturedPath)
            assertTrue(result.isSuccess)
        }

    @Test
    fun subscribeFeedSendsPost() =
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
                    respond(subscribeResponseJson(), HttpStatusCode.OK, jsonHeaders)
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val result = apiClient.subscribeFeed("https://example.com/feed.xml")

            assertEquals(HttpMethod.Post, capturedMethod)
            assertEquals("/api/v1/feeds/subscriptions", capturedPath)
            assertTrue(result.isSuccess)
        }

    @Test
    fun unsubscribeFeedSendsDelete() =
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
                    respond("", HttpStatusCode.NoContent, jsonHeaders)
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val result = apiClient.unsubscribeFeed("sub_01ABC")

            assertEquals(HttpMethod.Delete, capturedMethod)
            assertEquals("/api/v1/feeds/subscriptions/sub_01ABC", capturedPath)
            assertTrue(result.isSuccess)
        }

    @Test
    fun updateFeedSubscriptionSendsPatch() =
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
                    respond(feedSubscriptionJson(), HttpStatusCode.OK, jsonHeaders)
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val result =
                apiClient.updateFeedSubscription(
                    "sub_01ABC",
                    UpdateSubscriptionRequest(),
                )

            assertEquals(HttpMethod.Patch, capturedMethod)
            assertEquals("/api/v1/feeds/subscriptions/sub_01ABC", capturedPath)
            assertTrue(result.isSuccess)
        }

    @Test
    fun searchFeedSourcesSendsGetWithQuery() =
        runTest {
            val tokenStorage = InMemoryTokenStorage()
            tokenStorage.saveToken("test-token")
            tokenStorage.saveExpiresAt(FAR_FUTURE_EXPIRY)
            var capturedPath: String? = null
            var capturedQuery: String? = null
            val engine =
                MockEngine { request ->
                    capturedPath = request.url.encodedPath
                    capturedQuery = request.url.parameters["query"]
                    respond(feedSearchResponseJson(), HttpStatusCode.OK, jsonHeaders)
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val result = apiClient.searchFeedSources("tech news")

            assertEquals("/api/v1/feeds/search", capturedPath)
            assertNotNull(capturedQuery)
            assertTrue(result.isSuccess)
        }

    @Test
    fun retryFeedSubscriptionSendsPost() =
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
                    respond(feedSubscriptionJson(), HttpStatusCode.OK, jsonHeaders)
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val result = apiClient.retryFeedSubscription("sub_01ABC")

            assertEquals(HttpMethod.Post, capturedMethod)
            assertEquals("/api/v1/feeds/subscriptions/sub_01ABC/retry", capturedPath)
            assertTrue(result.isSuccess)
        }

    private fun feedItemWithStateJson() =
        """
        {
            "delivery_id": "fd_01ABC",
            "source_entry_id": "fse_01ABC",
            "object": "feed_delivery",
            "title": "Test Article",
            "url": "https://example.com/article",
            "source_id": "src_01ABC",
            "subscription_id": "sub_01ABC",
            "delivered_at": "2024-01-01T00:00:00Z",
            "document_id": null,
            "saved": false,
            "seen_at": null
        }
        """.trimIndent()

    private fun libraryEntryJson() =
        """
        {
            "library_entry_id": "lib_01ABC",
            "document_id": "doc_01ABC",
            "object": "library_entry",
            "title": "Test Article",
            "url": "https://example.com/article",
            "canonical_url": "https://example.com/article",
            "source": "feed",
            "document_type": "article",
            "triage_state": "inbox",
            "is_favorite": false,
            "is_shortlisted": false,
            "saved_at": "2026-01-01T00:00:00Z",
            "created_at": "2026-01-01T00:00:00Z",
            "updated_at": "2026-01-01T00:00:00Z"
        }
        """.trimIndent()

    private fun paginatedFeedItemsJson() =
        """
        {
            "data": [],
            "page": {"has_more": false}
        }
        """.trimIndent()

    private fun feedSubscriptionJson() =
        """
        {
            "id": "sub_01ABC",
            "object": "feed_subscription",
            "input_url": "https://example.com/feed.xml",
            "status": "active",
            "auto_save": false,
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-01T00:00:00Z",
            "source": {
                "id": "src_01ABC",
                "object": "feed_source",
                "name": "Example Feed",
                "url": "https://example.com/feed.xml",
                "poll_url": "https://example.com/feed.xml",
                "source_kind": "rss",
                "visibility": "public",
                "is_resolvable": true,
                "consecutive_failures": 0,
                "popularity": 0
            }
        }
        """.trimIndent()

    private fun paginatedSubscriptionsJson() =
        """
        {
            "data": [],
            "page": {"has_more": false}
        }
        """.trimIndent()

    private fun subscribeResponseJson() =
        """
        {
            "is_new": true,
            "subscription": ${feedSubscriptionJson()}
        }
        """.trimIndent()

    private fun feedSearchResponseJson() =
        """
        {
            "items": [
                {
                    "id": "src_01ABC",
                    "object": "feed_source",
                    "name": "Example Feed",
                    "url": "https://example.com/feed.xml",
                    "poll_url": "https://example.com/feed.xml",
                    "source_kind": "rss",
                    "visibility": "public",
                    "is_resolvable": true,
                    "consecutive_failures": 0,
                    "popularity": 0
                }
            ]
        }
        """.trimIndent()

    companion object {
        private const val FAR_FUTURE_EXPIRY = 4_102_444_800L
    }
}
