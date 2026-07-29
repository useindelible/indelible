package app.indelible.feed.model

import app.indelible.api.generated.models.FeedDeliveryResponse
import app.indelible.api.generated.models.PaginatedResponseFeedDeliveryResponse
import app.indelible.core.model.PageInfo
import kotlinx.datetime.Instant

data class FeedItemWithState(
    val id: String,
    val guid: String,
    val subscriptionId: String,
    val sourceId: String,
    val title: String,
    val url: String? = null,
    val author: String? = null,
    val excerpt: String? = null,
    val publishedAt: Instant? = null,
    val fetchedAt: Instant,
    val saved: Boolean,
    val state: String,
    val savedItemId: String? = null,
    val documentId: String? = null,
    val documentType: String? = null,
    val thumbnailUrl: String? = null,
    val leadImageUrl: String? = null,
)

typealias FeedItem = FeedItemWithState

data class PaginatedFeedItems(
    val `data`: List<FeedItemWithState>,
    val page: PageInfo,
)

fun FeedDeliveryResponse.toFeedItem(): FeedItemWithState =
    FeedItemWithState(
        id = deliveryId,
        guid = sourceEntryId,
        subscriptionId = subscriptionId,
        sourceId = sourceId,
        title = title,
        url = url,
        author = author,
        excerpt = excerpt,
        publishedAt = publishedAt,
        fetchedAt = deliveredAt,
        saved = saved,
        state = if (seenAt == null) "unseen" else "seen",
        savedItemId = if (saved) documentId else null,
        documentId = documentId,
        documentType = documentType,
        thumbnailUrl = thumbnailUrl,
        leadImageUrl = leadImageUrl,
    )

fun PaginatedResponseFeedDeliveryResponse.toPaginatedFeedItems(): PaginatedFeedItems =
    PaginatedFeedItems(
        data = data.map { it.toFeedItem() },
        page = page,
    )
