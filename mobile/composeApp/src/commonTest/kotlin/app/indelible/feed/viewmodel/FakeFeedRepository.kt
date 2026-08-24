package app.indelible.feed.viewmodel

import app.indelible.core.model.PageInfo
import app.indelible.feed.model.FeedItemWithState
import app.indelible.feed.model.FeedSource
import app.indelible.feed.model.FeedSubscription
import app.indelible.feed.model.OpmlImportResult
import app.indelible.feed.model.PaginatedFeedItems
import app.indelible.feed.model.PaginatedSubscriptions
import app.indelible.feed.model.UpdateSubscriptionRequest
import app.indelible.feed.repository.FeedRepository
import kotlinx.datetime.Instant

class FakeFeedRepository : FeedRepository {
    var listItemsResult: Result<PaginatedFeedItems> = Result.success(emptyPaginatedFeedItems())
    var markSeenResult: Result<Unit> = Result.success(Unit)
    var prepareDeliveryResult: Result<String> = Result.success("doc_fake")
    var saveToLibraryResult: Result<Unit> = Result.success(Unit)
    var markAllSeenResult: Result<Unit> = Result.success(Unit)
    var listSubscriptionsResult: Result<PaginatedSubscriptions> =
        Result.success(emptyPaginatedSubscriptions())
    var subscribeResult: Result<FeedSubscription> = Result.success(fakeSubscription())
    var unsubscribeResult: Result<Unit> = Result.success(Unit)
    var updateSubscriptionResult: Result<FeedSubscription> = Result.success(fakeSubscription())

    var lastListItemsState: String? = null
    var lastListItemsCursor: String? = null
    var listItemsCallCount = 0
    var lastMarkedSeenId: String? = null
    var lastPreparedDeliveryId: String? = null
    var lastSavedToLibraryId: String? = null
    var markAllSeenCallCount = 0

    override suspend fun listItems(
        state: String?,
        cursor: String?,
        limit: Int,
    ): Result<PaginatedFeedItems> {
        listItemsCallCount++
        lastListItemsState = state
        lastListItemsCursor = cursor
        return listItemsResult
    }

    override suspend fun markSeen(itemId: String): Result<Unit> {
        lastMarkedSeenId = itemId
        return markSeenResult
    }

    override suspend fun prepareDelivery(deliveryId: String): Result<String> {
        lastPreparedDeliveryId = deliveryId
        return prepareDeliveryResult
    }

    override suspend fun saveToLibrary(itemId: String): Result<Unit> {
        lastSavedToLibraryId = itemId
        return saveToLibraryResult
    }

    override suspend fun markAllSeen(subscriptionId: String?): Result<Unit> {
        markAllSeenCallCount++
        return markAllSeenResult
    }

    override suspend fun listSubscriptions(
        cursor: String?,
        limit: Int,
    ): Result<PaginatedSubscriptions> = listSubscriptionsResult

    override suspend fun subscribe(
        url: String,
        title: String?,
    ): Result<FeedSubscription> = subscribeResult

    override suspend fun unsubscribe(subscriptionId: String): Result<Unit> = unsubscribeResult

    override suspend fun updateSubscription(
        subscriptionId: String,
        request: UpdateSubscriptionRequest,
    ): Result<FeedSubscription> = updateSubscriptionResult

    var importOpmlResult: Result<OpmlImportResult> =
        Result.success(OpmlImportResult(created = 0, errors = emptyList(), skipped = 0))

    var importOpmlCallCount = 0
    var lastImportOpmlFileName: String? = null

    override suspend fun importOpml(
        fileBytes: ByteArray,
        fileName: String,
    ): Result<OpmlImportResult> {
        importOpmlCallCount++
        lastImportOpmlFileName = fileName
        return importOpmlResult
    }

    companion object {
        fun emptyPaginatedFeedItems() =
            PaginatedFeedItems(
                data = emptyList(),
                page = PageInfo(nextCursor = null, hasMore = false),
            )

        fun paginatedFeedItems(
            items: List<FeedItemWithState>,
            hasMore: Boolean = false,
            nextCursor: String? = null,
        ) = PaginatedFeedItems(
            data = items,
            page = PageInfo(nextCursor = nextCursor, hasMore = hasMore),
        )

        fun emptyPaginatedSubscriptions() =
            PaginatedSubscriptions(
                data = emptyList(),
                page = PageInfo(nextCursor = null, hasMore = false),
            )

        fun fakeFeedItem(id: String = "fd-1") =
            FeedItemWithState(
                id = id,
                guid = "fse-$id",
                subscriptionId = "sub-1",
                sourceId = "src-1",
                title = "Test Feed Item $id",
                url = "https://example.com/$id",
                author = "Author",
                excerpt = "Excerpt for $id",
                publishedAt = Instant.parse("2026-03-28T10:00:00Z"),
                fetchedAt = Instant.parse("2026-03-28T12:00:00Z"),
                saved = false,
                state = "unseen",
            )

        fun fakeSubscription(id: String = "sub-1") =
            FeedSubscription(
                id = id,
                inputUrl = "https://example.com/feed",
                titleOverride = null,
                autoSave = false,
                status = "active",
                source =
                    FeedSource(
                        id = "src-1",
                        name = "Example Blog",
                        url = "https://example.com/feed",
                        pollUrl = "https://example.com/feed",
                        domain = "example.com",
                        imageUrl = null,
                        consecutiveFailures = 0,
                        isResolvable = true,
                        popularity = 0,
                        sourceKind = "rss",
                        visibility = "public",
                    ),
                createdAt = Instant.parse("2026-03-20T10:00:00Z"),
                updatedAt = Instant.parse("2026-03-20T10:00:00Z"),
            )
    }
}
