package app.indelible.feed.repository

import app.indelible.feed.model.FeedSubscription
import app.indelible.feed.model.OpmlImportResult
import app.indelible.feed.model.PaginatedFeedItems
import app.indelible.feed.model.PaginatedSubscriptions
import app.indelible.feed.model.UpdateSubscriptionRequest

interface FeedRepository {
    suspend fun listItems(
        state: String? = null,
        cursor: String? = null,
        limit: Int = 50,
    ): Result<PaginatedFeedItems>

    suspend fun markSeen(itemId: String): Result<Unit>

    /** Prepares a delivery's readable document and returns its document id. */
    suspend fun prepareDelivery(deliveryId: String): Result<String>

    suspend fun saveToLibrary(itemId: String): Result<Unit>

    suspend fun markAllSeen(subscriptionId: String? = null): Result<Unit>

    suspend fun listSubscriptions(
        cursor: String? = null,
        limit: Int = 50,
    ): Result<PaginatedSubscriptions>

    suspend fun subscribe(
        url: String,
        title: String? = null,
    ): Result<FeedSubscription>

    suspend fun unsubscribe(subscriptionId: String): Result<Unit>

    suspend fun updateSubscription(
        subscriptionId: String,
        request: UpdateSubscriptionRequest,
    ): Result<FeedSubscription>

    suspend fun importOpml(
        fileBytes: ByteArray,
        fileName: String,
    ): Result<OpmlImportResult>
}
