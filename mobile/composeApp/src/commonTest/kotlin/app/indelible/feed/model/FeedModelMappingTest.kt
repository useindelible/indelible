package app.indelible.feed.model

import app.indelible.api.generated.models.FeedDeliveryResponse
import app.indelible.api.generated.models.FeedSourceResponse
import app.indelible.api.generated.models.FeedSubscriptionResponse
import app.indelible.api.generated.models.OpmlImportResponse
import app.indelible.api.generated.models.PageInfo
import app.indelible.api.generated.models.PaginatedResponseFeedDeliveryResponse
import kotlinx.datetime.Instant
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNull
import kotlin.test.assertTrue

class FeedModelMappingTest {
    @Test
    fun mapsGeneratedFeedDeliveryIntoAppModel() {
        val deliveredAt = Instant.parse("2026-03-28T12:00:00Z")
        val response =
            FeedDeliveryResponse(
                author = "Writer",
                deliveredAt = deliveredAt,
                deliveryId = "del_123",
                documentId = "doc_123",
                documentType = "article",
                excerpt = "Summary",
                leadImageUrl = "https://example.com/lead.jpg",
                `object` = "feed_delivery",
                publishedAt = Instant.parse("2026-03-28T10:00:00Z"),
                saved = true,
                seenAt = Instant.parse("2026-03-28T12:30:00Z"),
                sourceEntryId = "entry_123",
                sourceId = "src_123",
                subscriptionId = "sub_123",
                thumbnailUrl = "https://example.com/thumb.jpg",
                title = "Mapped Feed Item",
                url = "https://example.com/item",
            )

        val item = response.toFeedItem()

        assertEquals("del_123", item.id)
        assertEquals("entry_123", item.guid)
        assertEquals(deliveredAt, item.fetchedAt)
        assertEquals("seen", item.state)
        assertEquals("doc_123", item.savedItemId)
        assertEquals("article", item.documentType)
        assertEquals("https://example.com/thumb.jpg", item.thumbnailUrl)
    }

    @Test
    fun mapsFeedPagesWithoutExposingGeneratedPaginationWrapper() {
        val page = PageInfo(nextCursor = "next", hasMore = true)
        val response =
            PaginatedResponseFeedDeliveryResponse(
                data = listOf(feedDelivery("del_1")),
                page = page,
            )

        val paginated = response.toPaginatedFeedItems()

        assertEquals(page, paginated.page)
        assertEquals("del_1", paginated.data.single().id)
    }

    @Test
    fun mapsGeneratedFeedSubscriptionIntoAppModel() {
        val createdAt = Instant.parse("2026-03-20T10:00:00Z")
        val response =
            FeedSubscriptionResponse(
                autoSave = false,
                createdAt = createdAt,
                id = "sub_123",
                inputUrl = "https://example.com/feed",
                `object` = "feed_subscription",
                source =
                    FeedSourceResponse(
                        consecutiveFailures = 0,
                        domain = "example.com",
                        id = "src_123",
                        imageUrl = null,
                        isResolvable = true,
                        name = "Example",
                        `object` = "feed_source",
                        pollUrl = "https://example.com/feed",
                        popularity = 7,
                        sourceKind = "rss",
                        url = "https://example.com/feed",
                        visibility = "public",
                    ),
                status = "active",
                titleOverride = null,
                updatedAt = createdAt,
            )

        val subscription = response.toFeedSubscription()

        assertEquals("sub_123", subscription.id)
        assertFalse(subscription.autoSave)
        assertEquals("Example", subscription.source.name)
        assertEquals("example.com", subscription.source.domain)
        assertEquals(7, subscription.source.popularity)
    }

    @Test
    fun appFeedModelsDoNotRequireWireOnlyFields() {
        val item =
            FeedItemWithState(
                id = "del_local",
                guid = "entry_local",
                subscriptionId = "sub_local",
                sourceId = "src_local",
                title = "Local model",
                url = "https://example.com/local",
                fetchedAt = Instant.parse("2026-03-28T12:00:00Z"),
                saved = false,
                state = "unseen",
            )

        assertEquals("del_local", item.id)
        assertNull(item.savedItemId)
        assertTrue(item.title.isNotBlank())
    }

    @Test
    fun mapsAppUpdateSubscriptionRequestIntoGeneratedBody() {
        val request =
            UpdateSubscriptionRequest(
                autoSave = true,
                autoSaveCollectionId = "col_123",
                pollIntervalOverrideMinutes = 30,
                status = "paused",
                title = "Daily reads",
            )

        val body = request.toUpdateSubscriptionBody()

        assertEquals(true, body.autoSave)
        assertEquals("col_123", body.autoSaveCollectionId)
        assertEquals(30, body.pollIntervalOverrideMinutes)
        assertEquals("paused", body.status)
        assertEquals("Daily reads", body.title)
    }

    @Test
    fun mapsGeneratedOpmlImportResponseIntoAppModel() {
        val response = OpmlImportResponse(created = 2, errors = listOf("bad.xml"), skipped = 1)

        val result = response.toOpmlImportResult()

        assertEquals(2, result.created)
        assertEquals(listOf("bad.xml"), result.errors)
        assertEquals(1, result.skipped)
    }

    private fun feedDelivery(id: String): FeedDeliveryResponse =
        FeedDeliveryResponse(
            deliveredAt = Instant.parse("2026-03-28T12:00:00Z"),
            deliveryId = id,
            `object` = "feed_delivery",
            saved = false,
            sourceEntryId = "entry_$id",
            sourceId = "src_123",
            subscriptionId = "sub_123",
            title = "Item $id",
        )
}
