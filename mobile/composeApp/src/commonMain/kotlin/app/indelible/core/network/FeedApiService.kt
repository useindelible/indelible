package app.indelible.core.network

import app.indelible.api.generated.client.ApiV1FeedsDeliveriesClient
import app.indelible.api.generated.client.ApiV1FeedsDeliveriesMarkAllSeenClient
import app.indelible.api.generated.client.ApiV1FeedsDeliveriesPrepareClient
import app.indelible.api.generated.client.ApiV1FeedsDeliveriesSeenClient
import app.indelible.api.generated.client.ApiV1FeedsSearchClient
import app.indelible.api.generated.client.ApiV1FeedsSubscriptionsClient
import app.indelible.api.generated.client.ApiV1FeedsSubscriptionsRetryClient
import app.indelible.api.generated.client.ApiV1LibraryFromDeliveryClient
import app.indelible.api.generated.models.FeedDeliveryResponse
import app.indelible.api.generated.models.FeedSearchResponse
import app.indelible.api.generated.models.FeedSubscriptionResponse
import app.indelible.api.generated.models.MarkAllDeliveriesSeenBody
import app.indelible.api.generated.models.OpmlImportResponse
import app.indelible.api.generated.models.PaginatedResponseFeedDeliveryResponse
import app.indelible.api.generated.models.PaginatedResponseFeedSubscriptionResponse
import app.indelible.api.generated.models.PrepareDeliveryResponse
import app.indelible.api.generated.models.SaveFromDeliveryBody
import app.indelible.api.generated.models.SubscribeBody
import app.indelible.feed.model.UpdateSubscriptionRequest
import app.indelible.feed.model.toUpdateSubscriptionBody
import io.ktor.client.call.body
import io.ktor.client.request.forms.MultiPartFormDataContent
import io.ktor.client.request.forms.formData
import io.ktor.client.request.header
import io.ktor.client.request.post
import io.ktor.client.request.setBody
import io.ktor.client.statement.bodyAsText
import io.ktor.http.Headers
import io.ktor.http.HttpHeaders
import io.ktor.http.isSuccess

class FeedApiService(
    private val transport: AuthenticatedApiTransport,
) {
    suspend fun listFeedItems(
        state: String? = null,
        cursor: String? = null,
        limit: Int = 50,
    ): Result<PaginatedResponseFeedDeliveryResponse> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1FeedsDeliveriesClient(client).listFeedDeliveries(
                state = state,
                cursor = cursor,
                limit = limit,
                apiConfiguration = configuration,
            )
        }

    suspend fun markFeedItemSeen(itemId: String): Result<Unit> =
        transport
            .authenticatedRequest { client, configuration ->
                ApiV1FeedsDeliveriesSeenClient(client).markDeliverySeen(itemId, configuration)
            }.map { Unit }

    suspend fun prepareFeedDelivery(deliveryId: String): Result<PrepareDeliveryResponse> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1FeedsDeliveriesPrepareClient(client).prepareFeedDelivery(deliveryId, configuration)
        }

    suspend fun saveFeedItemToLibrary(itemId: String): Result<Unit> =
        transport
            .authenticatedRequest { client, configuration ->
                ApiV1LibraryFromDeliveryClient(client).saveFromDelivery(
                    saveFromDeliveryBody = SaveFromDeliveryBody(deliveryId = itemId),
                    apiConfiguration = configuration,
                )
            }.map { Unit }

    suspend fun markAllFeedItemsSeen(subscriptionId: String? = null): Result<Unit> =
        transport
            .authenticatedRequest { client, configuration ->
                ApiV1FeedsDeliveriesMarkAllSeenClient(client).markAllDeliveriesSeen(
                    markAllDeliveriesSeenBody = MarkAllDeliveriesSeenBody(subscriptionId = subscriptionId),
                    apiConfiguration = configuration,
                )
            }.map { Unit }

    suspend fun listFeedSubscriptions(
        cursor: String? = null,
        limit: Int = 50,
    ): Result<PaginatedResponseFeedSubscriptionResponse> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1FeedsSubscriptionsClient(client).listSubscriptions(
                cursor = cursor,
                limit = limit,
                apiConfiguration = configuration,
            )
        }

    suspend fun subscribeFeed(
        url: String,
        title: String? = null,
    ): Result<FeedSubscriptionResponse> =
        transport
            .authenticatedRequest { client, configuration ->
                ApiV1FeedsSubscriptionsClient(client).subscribe(
                    subscribeBody = SubscribeBody(url = url, title = title),
                    apiConfiguration = configuration,
                )
            }.map { it.subscription }

    suspend fun unsubscribeFeed(subscriptionId: String): Result<Unit> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1FeedsSubscriptionsClient(client).unsubscribe(subscriptionId, configuration)
        }

    suspend fun importOpml(
        fileBytes: ByteArray,
        fileName: String,
    ): Result<OpmlImportResponse> =
        transport.directAuthenticatedRequest { client, baseUrl, token ->
            val response =
                client.post("$baseUrl/api/v1/feeds/subscriptions/opml") {
                    header("Authorization", "Bearer $token")
                    setBody(
                        MultiPartFormDataContent(
                            formData {
                                append(
                                    "file",
                                    fileBytes,
                                    Headers.build {
                                        append(HttpHeaders.ContentDisposition, "filename=\"$fileName\"")
                                        append(HttpHeaders.ContentType, "application/xml")
                                    },
                                )
                            },
                        ),
                    )
                }
            if (!response.status.isSuccess()) {
                throw ApiException(response.status.value, response.bodyAsText())
            }
            response.body<OpmlImportResponse>()
        }

    suspend fun updateFeedSubscription(
        subscriptionId: String,
        request: UpdateSubscriptionRequest,
    ): Result<FeedSubscriptionResponse> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1FeedsSubscriptionsClient(client).updateSubscription(
                updateSubscriptionBody = request.toUpdateSubscriptionBody(),
                id = subscriptionId,
                apiConfiguration = configuration,
            )
        }

    suspend fun getFeedItem(id: String): Result<FeedDeliveryResponse> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1FeedsDeliveriesClient(client).getFeedDelivery(id, configuration)
        }

    suspend fun searchFeedSources(
        query: String,
        limit: Int = 20,
    ): Result<FeedSearchResponse> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1FeedsSearchClient(client).searchSources(
                query = query,
                limit = limit,
                apiConfiguration = configuration,
            )
        }

    suspend fun retryFeedSubscription(id: String): Result<Unit> =
        transport
            .authenticatedRequest { client, configuration ->
                ApiV1FeedsSubscriptionsRetryClient(client).retrySubscription(id, configuration)
            }.map { Unit }
}
