package app.indelible.feed.model

import app.indelible.api.generated.models.FeedSourceResponse
import app.indelible.api.generated.models.FeedSubscriptionResponse
import app.indelible.api.generated.models.PaginatedResponseFeedSubscriptionResponse
import app.indelible.api.generated.models.UpdateSubscriptionBody
import app.indelible.core.model.PageInfo
import kotlinx.datetime.Instant

data class FeedSource(
    val id: String,
    val name: String,
    val url: String,
    val pollUrl: String,
    val domain: String? = null,
    val imageUrl: String? = null,
    val consecutiveFailures: Int,
    val isResolvable: Boolean,
    val popularity: Int,
    val sourceKind: String,
    val visibility: String,
    val description: String? = null,
    val lastEntryAddedAt: Instant? = null,
    val lastError: String? = null,
    val lastPolledAt: Instant? = null,
    val nextPollAt: Instant? = null,
    val provider: String? = null,
    val siteUrl: String? = null,
)

data class FeedSubscription(
    val id: String,
    val inputUrl: String,
    val titleOverride: String? = null,
    val autoSave: Boolean,
    val status: String,
    val source: FeedSource,
    val createdAt: Instant,
    val updatedAt: Instant,
    val autoSaveCollectionId: String? = null,
    val pollIntervalOverrideMinutes: Int? = null,
)

data class PaginatedSubscriptions(
    val `data`: List<FeedSubscription>,
    val page: PageInfo,
)

data class UpdateSubscriptionRequest(
    val autoSave: Boolean? = null,
    val autoSaveCollectionId: String? = null,
    val pollIntervalOverrideMinutes: Int? = null,
    val status: String? = null,
    val title: String? = null,
)

fun FeedSourceResponse.toFeedSource(): FeedSource =
    FeedSource(
        id = id,
        name = name,
        url = url,
        pollUrl = pollUrl,
        domain = domain,
        imageUrl = imageUrl,
        consecutiveFailures = consecutiveFailures,
        isResolvable = isResolvable,
        popularity = popularity,
        sourceKind = sourceKind,
        visibility = visibility,
        description = description,
        lastEntryAddedAt = lastEntryAddedAt,
        lastError = lastError,
        lastPolledAt = lastPolledAt,
        nextPollAt = nextPollAt,
        provider = provider,
        siteUrl = siteUrl,
    )

fun FeedSubscriptionResponse.toFeedSubscription(): FeedSubscription =
    FeedSubscription(
        id = id,
        inputUrl = inputUrl,
        titleOverride = titleOverride,
        autoSave = autoSave,
        status = status,
        source = source.toFeedSource(),
        createdAt = createdAt,
        updatedAt = updatedAt,
        autoSaveCollectionId = autoSaveCollectionId,
        pollIntervalOverrideMinutes = pollIntervalOverrideMinutes,
    )

fun PaginatedResponseFeedSubscriptionResponse.toPaginatedSubscriptions(): PaginatedSubscriptions =
    PaginatedSubscriptions(
        data = data.map { it.toFeedSubscription() },
        page = page,
    )

fun UpdateSubscriptionRequest.toUpdateSubscriptionBody(): UpdateSubscriptionBody =
    UpdateSubscriptionBody(
        autoSave = autoSave,
        autoSaveCollectionId = autoSaveCollectionId,
        pollIntervalOverrideMinutes = pollIntervalOverrideMinutes,
        status = status,
        title = title,
    )
