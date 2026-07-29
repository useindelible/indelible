package app.indelible.feed.repository

import app.indelible.core.network.FeedApiService
import app.indelible.feed.model.FeedSubscription
import app.indelible.feed.model.OpmlImportResult
import app.indelible.feed.model.PaginatedFeedItems
import app.indelible.feed.model.PaginatedSubscriptions
import app.indelible.feed.model.UpdateSubscriptionRequest
import app.indelible.feed.model.toFeedSubscription
import app.indelible.feed.model.toOpmlImportResult
import app.indelible.feed.model.toPaginatedFeedItems
import app.indelible.feed.model.toPaginatedSubscriptions

class ApiFeedRepository(
    private val feedApiService: FeedApiService,
) : FeedRepository {
    override suspend fun listItems(
        state: String?,
        cursor: String?,
        limit: Int,
    ): Result<PaginatedFeedItems> = feedApiService.listFeedItems(state, cursor, limit).map { it.toPaginatedFeedItems() }

    override suspend fun markSeen(itemId: String): Result<Unit> = feedApiService.markFeedItemSeen(itemId)

    override suspend fun prepareDelivery(deliveryId: String): Result<String> =
        feedApiService.prepareFeedDelivery(deliveryId).map { it.documentId }

    override suspend fun saveToLibrary(itemId: String): Result<Unit> = feedApiService.saveFeedItemToLibrary(itemId)

    override suspend fun markAllSeen(subscriptionId: String?): Result<Unit> = feedApiService.markAllFeedItemsSeen(subscriptionId)

    override suspend fun listSubscriptions(
        cursor: String?,
        limit: Int,
    ): Result<PaginatedSubscriptions> = feedApiService.listFeedSubscriptions(cursor, limit).map { it.toPaginatedSubscriptions() }

    override suspend fun subscribe(
        url: String,
        title: String?,
    ): Result<FeedSubscription> = feedApiService.subscribeFeed(url, title).map { it.toFeedSubscription() }

    override suspend fun unsubscribe(subscriptionId: String): Result<Unit> = feedApiService.unsubscribeFeed(subscriptionId)

    override suspend fun updateSubscription(
        subscriptionId: String,
        request: UpdateSubscriptionRequest,
    ): Result<FeedSubscription> = feedApiService.updateFeedSubscription(subscriptionId, request).map { it.toFeedSubscription() }

    override suspend fun importOpml(
        fileBytes: ByteArray,
        fileName: String,
    ): Result<OpmlImportResult> = feedApiService.importOpml(fileBytes, fileName).map { it.toOpmlImportResult() }
}
