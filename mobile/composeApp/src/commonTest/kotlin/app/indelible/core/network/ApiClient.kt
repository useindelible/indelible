package app.indelible.core.network

import app.indelible.api.generated.models.ApiTokenResponse
import app.indelible.api.generated.models.ArchivalSettingsResponse
import app.indelible.api.generated.models.CollectionResponse
import app.indelible.api.generated.models.CreateApiTokenRequest
import app.indelible.api.generated.models.CreateApiTokenResponse
import app.indelible.api.generated.models.CreateCollectionBody
import app.indelible.api.generated.models.CreateMilaPromptPresetBody
import app.indelible.api.generated.models.CreateMilaSessionBody
import app.indelible.api.generated.models.CreateSmartListBody
import app.indelible.api.generated.models.CreateTagBody
import app.indelible.api.generated.models.DocumentNoteResponse
import app.indelible.api.generated.models.DocumentReaderResponse
import app.indelible.api.generated.models.FeedDeliveryResponse
import app.indelible.api.generated.models.FeedSearchResponse
import app.indelible.api.generated.models.FeedSubscriptionResponse
import app.indelible.api.generated.models.HighlightListResponse
import app.indelible.api.generated.models.HighlightNoteResponse
import app.indelible.api.generated.models.HighlightWithNoteResponse
import app.indelible.api.generated.models.HomeDashboardResponse
import app.indelible.api.generated.models.HomeSettingsResponse
import app.indelible.api.generated.models.ImportJobStatusResponse
import app.indelible.api.generated.models.ImportUploadResponse
import app.indelible.api.generated.models.LibraryCountResponse
import app.indelible.api.generated.models.LibraryEntryResponse
import app.indelible.api.generated.models.MergeTagsBody
import app.indelible.api.generated.models.MilaConfigResponse
import app.indelible.api.generated.models.MilaConversationResponse
import app.indelible.api.generated.models.MilaPromptPresetResponse
import app.indelible.api.generated.models.MilaPromptPresetsResponse
import app.indelible.api.generated.models.MilaSessionListResponse
import app.indelible.api.generated.models.MilaSessionResponse
import app.indelible.api.generated.models.NotificationsSettingsResponse
import app.indelible.api.generated.models.OAuthProvidersResponse
import app.indelible.api.generated.models.OpmlImportResponse
import app.indelible.api.generated.models.PaginatedResponseCollectionResponse
import app.indelible.api.generated.models.PaginatedResponseFeedDeliveryResponse
import app.indelible.api.generated.models.PaginatedResponseFeedSubscriptionResponse
import app.indelible.api.generated.models.PaginatedResponseHighlightResponse
import app.indelible.api.generated.models.PaginatedResponseLibraryEntryResponse
import app.indelible.api.generated.models.PaginatedResponseSmartListResponse
import app.indelible.api.generated.models.PreferencesSettingsResponse
import app.indelible.api.generated.models.PrepareDeliveryResponse
import app.indelible.api.generated.models.RecentHighlightsResponse
import app.indelible.api.generated.models.SmartListResponse
import app.indelible.api.generated.models.TagResponse
import app.indelible.api.generated.models.TestMilaConfigBody
import app.indelible.api.generated.models.TestMilaConfigResponse
import app.indelible.api.generated.models.UpdateCollectionBody
import app.indelible.api.generated.models.UpdateHomeSettingsBody
import app.indelible.api.generated.models.UpdateMilaPromptPresetBody
import app.indelible.api.generated.models.UpdateSmartListBody
import app.indelible.api.generated.models.UpdateTagBody
import app.indelible.api.generated.models.UpsertMilaConfigBody
import app.indelible.core.model.AuthResponse
import app.indelible.core.model.AuthUser
import app.indelible.core.model.OnboardingStatusResponse
import app.indelible.core.model.SaveItemRequest
import app.indelible.core.model.StepData
import app.indelible.core.storage.TokenStorage
import app.indelible.feed.model.UpdateSubscriptionRequest
import app.indelible.reader.model.AssetWithUrlResponse
import app.indelible.reader.model.CreateHighlightRequest
import app.indelible.search.model.PaginatedSearchResults
import app.indelible.search.model.RecentSearch
import app.indelible.search.model.SearchSuggestion
import io.ktor.client.engine.HttpClientEngine

/** Test-only compatibility surface that exercises the production domain services. */
class ApiClient(
    tokenStorage: TokenStorage,
    onUnauthorized: suspend () -> Unit = {},
    engine: HttpClientEngine? = null,
) {
    val transport = AuthenticatedApiTransport(tokenStorage, onUnauthorized, engine)
    val authApiService = AuthApiService(transport)
    val accountApiService = AccountApiService(transport)
    val onboardingApiService = OnboardingApiService(transport)
    val libraryApiService = LibraryApiService(transport)
    val feedApiService = FeedApiService(transport)
    val readerApiService = ReaderApiService(transport)
    val collectionsApiService = CollectionsApiService(transport)
    val searchApiService = SearchApiService(transport)
    val tagsApiService = TagsApiService(transport)
    val settingsApiService = SettingsApiService(transport)
    val homeApiService = HomeApiService(transport)
    val milaApiService = MilaApiService(transport)
    val trashApiService = TrashApiService(transport)
    val importApiService = ImportApiService(transport)

    suspend fun login(
        email: String,
        password: String,
    ): Result<AuthResponse> = authApiService.login(email, password)

    suspend fun register(
        name: String,
        email: String,
        password: String,
    ): Result<AuthResponse> = authApiService.register(name, email, password)

    suspend fun forgotPassword(email: String): Result<Unit> = authApiService.forgotPassword(email)

    suspend fun resetPassword(
        token: String,
        newPassword: String,
    ): Result<Unit> = authApiService.resetPassword(token, newPassword)

    suspend fun logout(): Result<Unit> = authApiService.logout()

    suspend fun getSession(): Result<AuthUser> = accountApiService.getSession()

    suspend fun resendVerification(): Result<Unit> = authApiService.resendVerification()

    suspend fun getOAuthProviders(): Result<OAuthProvidersResponse> = authApiService.getOAuthProviders()

    suspend fun nativeOAuthStartUrl(
        providerId: String,
        codeChallenge: String,
        appState: String,
    ): String = authApiService.nativeOAuthStartUrl(providerId, codeChallenge, appState)

    suspend fun exchangeNativeOAuthCode(
        code: String,
        codeVerifier: String,
    ): Result<NativeOAuthTokenResponse> = authApiService.exchangeNativeOAuthCode(code, codeVerifier)

    suspend fun updateProfile(displayName: String): Result<AuthUser> = accountApiService.updateProfile(displayName)

    suspend fun deleteAccount(confirmation: String): Result<Unit> = accountApiService.deleteAccount(confirmation)

    suspend fun fetchAvatarBytes(avatarUrl: String): Result<ByteArray> = accountApiService.fetchAvatarBytes(avatarUrl)

    suspend fun resolveImageRequest(url: String): ResolvedImageRequest = transport.resolveImageRequest(url)

    suspend fun changeEmail(
        newEmail: String,
        password: String,
    ): Result<Unit> = accountApiService.changeEmail(newEmail, password)

    suspend fun changePassword(
        currentPassword: String,
        newPassword: String,
    ): Result<Unit> = accountApiService.changePassword(currentPassword, newPassword)

    suspend fun getOnboardingStatus(): Result<OnboardingStatusResponse> = onboardingApiService.getOnboardingStatus()

    suspend fun completeOnboardingStep(
        step: Int,
        data: StepData = StepData(),
    ): Result<OnboardingStatusResponse> = onboardingApiService.completeOnboardingStep(step, data)

    suspend fun skipOnboarding(): Result<OnboardingStatusResponse> = onboardingApiService.skipOnboarding()

    suspend fun saveItem(request: SaveItemRequest): Result<LibraryEntryResponse> = libraryApiService.saveItem(request)

    suspend fun listItems(
        triageState: String? = null,
        itemType: String? = null,
        cursor: String? = null,
        limit: Int = 50,
    ): Result<PaginatedResponseLibraryEntryResponse> = libraryApiService.listItems(triageState, itemType, cursor, limit)

    suspend fun getItem(itemId: String): Result<LibraryEntryResponse> = libraryApiService.getItem(itemId)

    suspend fun triageItem(
        itemId: String,
        state: String,
    ): Result<LibraryEntryResponse> = libraryApiService.triageItem(itemId, state)

    suspend fun toggleFavorite(itemId: String): Result<LibraryEntryResponse> = libraryApiService.toggleFavorite(itemId)

    suspend fun toggleShortlist(itemId: String): Result<LibraryEntryResponse> = libraryApiService.toggleShortlist(itemId)

    suspend fun deleteItem(itemId: String): Result<Unit> = libraryApiService.deleteItem(itemId)

    suspend fun rearchiveItem(itemId: String): Result<LibraryEntryResponse> = libraryApiService.rearchiveItem(itemId)

    suspend fun getDocumentReader(documentId: String): Result<DocumentReaderResponse> = readerApiService.getDocumentReader(documentId)

    suspend fun getAssetWithUrl(
        itemId: String,
        assetKind: String,
    ): Result<AssetWithUrlResponse> = readerApiService.getAssetWithUrl(itemId, assetKind)

    suspend fun streamAsset(
        itemId: String,
        assetKind: String,
    ): Result<String> = readerApiService.streamAsset(itemId, assetKind)

    suspend fun updateProgress(
        itemId: String,
        progressPercent: Float,
    ): Result<Unit> = readerApiService.updateProgress(itemId, progressPercent)

    suspend fun listHighlights(itemId: String): Result<HighlightListResponse> = readerApiService.listHighlights(itemId)

    suspend fun createHighlight(
        itemId: String,
        request: CreateHighlightRequest,
    ): Result<HighlightWithNoteResponse> = readerApiService.createHighlight(itemId, request)

    suspend fun deleteHighlight(highlightId: String): Result<Unit> = readerApiService.deleteHighlight(highlightId)

    suspend fun patchHighlight(
        highlightId: String,
        color: String,
    ): Result<HighlightWithNoteResponse> = readerApiService.patchHighlight(highlightId, color)

    suspend fun upsertHighlightNote(
        highlightId: String,
        body: String,
    ): Result<HighlightNoteResponse> = readerApiService.upsertHighlightNote(highlightId, body)

    suspend fun deleteHighlightNote(highlightId: String): Result<Unit> = readerApiService.deleteHighlightNote(highlightId)

    suspend fun setHighlightTags(
        highlightId: String,
        tags: List<String>,
    ): Result<List<String>> = readerApiService.setHighlightTags(highlightId, tags)

    suspend fun getItemNote(itemId: String): Result<DocumentNoteResponse?> = readerApiService.getItemNote(itemId)

    suspend fun upsertItemNote(
        itemId: String,
        noteBody: String,
    ): Result<DocumentNoteResponse> = readerApiService.upsertItemNote(itemId, noteBody)

    suspend fun getItemTags(itemId: String): Result<List<String>> = readerApiService.getItemTags(itemId)

    suspend fun setItemTags(
        itemId: String,
        tags: List<String>,
    ): Result<List<String>> = readerApiService.setItemTags(itemId, tags)

    suspend fun listFeedItems(
        state: String? = null,
        cursor: String? = null,
        limit: Int = 50,
    ): Result<PaginatedResponseFeedDeliveryResponse> = feedApiService.listFeedItems(state, cursor, limit)

    suspend fun markFeedItemSeen(itemId: String): Result<Unit> = feedApiService.markFeedItemSeen(itemId)

    suspend fun prepareFeedDelivery(deliveryId: String): Result<PrepareDeliveryResponse> = feedApiService.prepareFeedDelivery(deliveryId)

    suspend fun saveFeedItemToLibrary(itemId: String): Result<Unit> = feedApiService.saveFeedItemToLibrary(itemId)

    suspend fun markAllFeedItemsSeen(subscriptionId: String? = null): Result<Unit> = feedApiService.markAllFeedItemsSeen(subscriptionId)

    suspend fun listFeedSubscriptions(
        cursor: String? = null,
        limit: Int = 50,
    ): Result<PaginatedResponseFeedSubscriptionResponse> = feedApiService.listFeedSubscriptions(cursor, limit)

    suspend fun subscribeFeed(
        url: String,
        title: String? = null,
    ): Result<FeedSubscriptionResponse> = feedApiService.subscribeFeed(url, title)

    suspend fun unsubscribeFeed(subscriptionId: String): Result<Unit> = feedApiService.unsubscribeFeed(subscriptionId)

    suspend fun importOpml(
        fileBytes: ByteArray,
        fileName: String,
    ): Result<OpmlImportResponse> = feedApiService.importOpml(fileBytes, fileName)

    suspend fun updateFeedSubscription(
        subscriptionId: String,
        request: UpdateSubscriptionRequest,
    ): Result<FeedSubscriptionResponse> = feedApiService.updateFeedSubscription(subscriptionId, request)

    suspend fun getFeedItem(id: String): Result<FeedDeliveryResponse> = feedApiService.getFeedItem(id)

    suspend fun searchFeedSources(
        query: String,
        limit: Int = 20,
    ): Result<FeedSearchResponse> = feedApiService.searchFeedSources(query, limit)

    suspend fun retryFeedSubscription(id: String): Result<Unit> = feedApiService.retryFeedSubscription(id)

    suspend fun search(
        query: String,
        cursor: String? = null,
        limit: Int = 20,
    ): Result<PaginatedSearchResults> = searchApiService.search(query, cursor, limit)

    suspend fun suggestions(
        query: String,
        limit: Int = 8,
    ): Result<List<SearchSuggestion>> = searchApiService.suggestions(query, limit)

    suspend fun listRecentSearches(limit: Int = 20): Result<List<RecentSearch>> = searchApiService.listRecentSearches(limit)

    suspend fun deleteRecentSearch(id: String): Result<Unit> = searchApiService.deleteRecentSearch(id)

    suspend fun clearRecentSearches(): Result<Unit> = searchApiService.clearRecentSearches()

    suspend fun listCollections(
        cursor: String? = null,
        limit: Int = 50,
    ): Result<PaginatedResponseCollectionResponse> = collectionsApiService.listCollections(cursor, limit)

    suspend fun createCollection(body: CreateCollectionBody): Result<CollectionResponse> = collectionsApiService.createCollection(body)

    suspend fun getCollection(id: String): Result<CollectionResponse> = collectionsApiService.getCollection(id)

    suspend fun updateCollection(
        id: String,
        body: UpdateCollectionBody,
    ): Result<CollectionResponse> = collectionsApiService.updateCollection(id, body)

    suspend fun deleteCollection(id: String): Result<Unit> = collectionsApiService.deleteCollection(id)

    suspend fun listCollectionChildren(
        id: String,
        cursor: String? = null,
        limit: Int = 50,
    ): Result<PaginatedResponseCollectionResponse> = collectionsApiService.listCollectionChildren(id, cursor, limit)

    suspend fun listCollectionItems(
        id: String,
        cursor: String? = null,
        limit: Int = 50,
    ): Result<PaginatedResponseLibraryEntryResponse> = collectionsApiService.listCollectionItems(id, cursor, limit)

    suspend fun addItemToCollection(
        collectionId: String,
        itemId: String,
    ): Result<Unit> = collectionsApiService.addItemToCollection(collectionId, itemId)

    suspend fun removeItemFromCollection(
        collectionId: String,
        itemId: String,
    ): Result<Unit> = collectionsApiService.removeItemFromCollection(collectionId, itemId)

    suspend fun listSmartLists(
        cursor: String? = null,
        limit: Int = 50,
    ): Result<PaginatedResponseSmartListResponse> = collectionsApiService.listSmartLists(cursor, limit)

    suspend fun createSmartList(body: CreateSmartListBody): Result<SmartListResponse> = collectionsApiService.createSmartList(body)

    suspend fun getSmartList(id: String): Result<SmartListResponse> = collectionsApiService.getSmartList(id)

    suspend fun updateSmartList(
        id: String,
        body: UpdateSmartListBody,
    ): Result<SmartListResponse> = collectionsApiService.updateSmartList(id, body)

    suspend fun deleteSmartList(id: String): Result<Unit> = collectionsApiService.deleteSmartList(id)

    suspend fun listSmartListItems(
        id: String,
        cursor: String? = null,
        limit: Int = 50,
    ): Result<PaginatedResponseLibraryEntryResponse> = collectionsApiService.listSmartListItems(id, cursor, limit)

    suspend fun pinSmartList(
        id: String,
        isPinned: Boolean,
    ): Result<SmartListResponse> = collectionsApiService.pinSmartList(id, isPinned)

    suspend fun listTags(
        scope: String? = null,
        limit: Int = 100,
    ): Result<List<TagResponse>> = tagsApiService.listTags(scope, limit)

    suspend fun createTag(body: CreateTagBody): Result<TagResponse> = tagsApiService.createTag(body)

    suspend fun mergeTags(body: MergeTagsBody): Result<TagResponse> = tagsApiService.mergeTags(body)

    suspend fun getTag(id: String): Result<TagResponse> = tagsApiService.getTag(id)

    suspend fun updateTag(
        id: String,
        body: UpdateTagBody,
    ): Result<TagResponse> = tagsApiService.updateTag(id, body)

    suspend fun deleteTag(id: String): Result<Unit> = tagsApiService.deleteTag(id)

    suspend fun listTagHighlights(
        id: String,
        cursor: String? = null,
        limit: Int = 20,
    ): Result<PaginatedResponseHighlightResponse> = tagsApiService.listTagHighlights(id, cursor, limit)

    suspend fun listTagItems(
        id: String,
        cursor: String? = null,
        limit: Int = 50,
    ): Result<PaginatedResponseLibraryEntryResponse> = tagsApiService.listTagItems(id, cursor, limit)

    suspend fun listRecentHighlights(limit: Int = 20): Result<RecentHighlightsResponse> = tagsApiService.listRecentHighlights(limit)

    suspend fun getHighlightTags(highlightId: String): Result<List<String>> = tagsApiService.getHighlightTags(highlightId)

    suspend fun getArchivalSettings(): Result<ArchivalSettingsResponse> = settingsApiService.getArchivalSettings()

    suspend fun updateArchivalSettings(body: ArchivalSettingsResponse): Result<ArchivalSettingsResponse> =
        settingsApiService.updateArchivalSettings(body)

    suspend fun getNotificationsSettings(): Result<NotificationsSettingsResponse> = settingsApiService.getNotificationsSettings()

    suspend fun updateNotificationsSettings(body: NotificationsSettingsResponse): Result<NotificationsSettingsResponse> =
        settingsApiService.updateNotificationsSettings(body)

    suspend fun getPreferences(): Result<PreferencesSettingsResponse> = settingsApiService.getPreferences()

    suspend fun updatePreferences(body: PreferencesSettingsResponse): Result<PreferencesSettingsResponse> =
        settingsApiService.updatePreferences(body)

    suspend fun getHomeDashboard(): Result<HomeDashboardResponse> = homeApiService.getHomeDashboard()

    suspend fun getHomeSettings(): Result<HomeSettingsResponse> = homeApiService.getHomeSettings()

    suspend fun updateHomeSettings(body: UpdateHomeSettingsBody): Result<HomeSettingsResponse> = homeApiService.updateHomeSettings(body)

    suspend fun listApiTokens(): Result<List<ApiTokenResponse>> = accountApiService.listApiTokens()

    suspend fun createApiToken(body: CreateApiTokenRequest): Result<CreateApiTokenResponse> = accountApiService.createApiToken(body)

    suspend fun deleteApiToken(tokenId: String): Result<Unit> = accountApiService.deleteApiToken(tokenId)

    suspend fun getMilaConfig(): Result<MilaConfigResponse> = milaApiService.getConfig()

    suspend fun upsertMilaConfig(body: UpsertMilaConfigBody): Result<MilaConfigResponse> = milaApiService.upsertConfig(body)

    suspend fun reindexMilaConfig(body: UpsertMilaConfigBody): Result<MilaConfigResponse> = milaApiService.reindexConfig(body)

    suspend fun testMilaConfig(body: TestMilaConfigBody): Result<TestMilaConfigResponse> = milaApiService.testConfig(body)

    suspend fun getPromptPresets(): Result<MilaPromptPresetsResponse> = milaApiService.getPromptPresets()

    suspend fun createPromptPreset(body: CreateMilaPromptPresetBody): Result<MilaPromptPresetResponse> =
        milaApiService.createPromptPreset(body)

    suspend fun updatePromptPreset(
        presetId: String,
        body: UpdateMilaPromptPresetBody,
    ): Result<MilaPromptPresetResponse> = milaApiService.updatePromptPreset(presetId, body)

    suspend fun deletePromptPreset(presetId: String): Result<Unit> = milaApiService.deletePromptPreset(presetId)

    suspend fun listMilaSessions(limit: Int = 50): Result<MilaSessionListResponse> = milaApiService.listSessions(limit)

    suspend fun createMilaSession(body: CreateMilaSessionBody): Result<MilaSessionResponse> = milaApiService.createSession(body)

    suspend fun getMilaMessages(sessionId: String): Result<MilaConversationResponse> = milaApiService.getMessages(sessionId)

    suspend fun listTrash(
        cursor: String? = null,
        limit: Int = 50,
    ): Result<PaginatedResponseLibraryEntryResponse> = trashApiService.listTrash(cursor, limit)

    suspend fun getLibraryCount(): Result<LibraryCountResponse> = libraryApiService.getLibraryCount()

    suspend fun emptyTrash(): Result<Unit> = trashApiService.emptyTrash()

    suspend fun restoreItem(itemId: String): Result<LibraryEntryResponse> = trashApiService.restoreItem(itemId)

    suspend fun permanentlyDeleteItem(itemId: String): Result<Unit> = trashApiService.permanentlyDeleteItem(itemId)

    suspend fun uploadImport(
        sourceSlug: String,
        fileBytes: ByteArray,
        fileName: String,
        contentType: String,
    ): Result<ImportUploadResponse> = importApiService.uploadImport(sourceSlug, fileBytes, fileName, contentType)

    suspend fun getImport(importJobId: String): Result<ImportJobStatusResponse> = importApiService.getImport(importJobId)

    suspend fun rollbackImport(importJobId: String): Result<Unit> = importApiService.rollbackImport(importJobId)

    fun close() = transport.close()

    companion object {
        const val DEFAULT_SERVER_URL = AuthenticatedApiTransport.DEFAULT_SERVER_URL
    }
}
