package app.indelible.core.di

import app.indelible.auth.oauth.OAuthBrowserLauncher
import app.indelible.auth.repository.ApiAuthRepository
import app.indelible.auth.repository.AuthRepository
import app.indelible.auth.server.HttpServerHealthChecker
import app.indelible.auth.server.ServerHealthChecker
import app.indelible.auth.viewmodel.AuthViewModel
import app.indelible.auth.viewmodel.ConnectServerViewModel
import app.indelible.auth.viewmodel.OfflineAccount
import app.indelible.collections.repository.ApiCollectionsRepository
import app.indelible.collections.repository.CollectionsRepository
import app.indelible.core.config.ServerBuildConfig
import app.indelible.core.network.AccountApiService
import app.indelible.core.network.AuthApiService
import app.indelible.core.network.AuthenticatedApiTransport
import app.indelible.core.network.CollectionsApiService
import app.indelible.core.network.FeedApiService
import app.indelible.core.network.HomeApiService
import app.indelible.core.network.ImportApiService
import app.indelible.core.network.LibraryApiService
import app.indelible.core.network.MilaApiService
import app.indelible.core.network.OnboardingApiService
import app.indelible.core.network.ReaderApiService
import app.indelible.core.network.SearchApiService
import app.indelible.core.network.ServerCapabilities
import app.indelible.core.network.SettingsApiService
import app.indelible.core.network.TagsApiService
import app.indelible.core.network.TrashApiService
import app.indelible.core.offline.ApiOutboxSender
import app.indelible.core.offline.ConnectivityObserver
import app.indelible.core.offline.DownloadManager
import app.indelible.core.offline.OfflineCopies
import app.indelible.core.offline.OfflineFilesRoot
import app.indelible.core.offline.OfflineSetFetcher
import app.indelible.core.offline.OfflineStore
import app.indelible.core.offline.OutboxSender
import app.indelible.core.offline.OutboxWorker
import app.indelible.core.offline.ScopePurger
import app.indelible.core.offline.SessionRegistry
import app.indelible.core.offline.SessionTransitions
import app.indelible.core.offline.SqlDelightOfflineStore
import app.indelible.core.storage.TokenStorage
import app.indelible.core.storage.UserPreferencesStorage
import app.indelible.db.DatabaseDriverFactory
import app.indelible.db.OfflineDatabase
import app.indelible.feed.repository.ApiFeedRepository
import app.indelible.feed.repository.FeedRepository
import app.indelible.feed.viewmodel.AddFeedViewModel
import app.indelible.feed.viewmodel.FeedManagementViewModel
import app.indelible.feed.viewmodel.FeedViewModel
import app.indelible.home.repository.ApiHomeRepository
import app.indelible.home.repository.HomeRepository
import app.indelible.home.viewmodel.OfflineShelfViewModel
import app.indelible.library.repository.ApiLibraryRepository
import app.indelible.library.repository.LibraryRepository
import app.indelible.library.viewmodel.LibraryViewModel
import app.indelible.mila.data.MilaRepository
import app.indelible.offline.viewmodel.OfflineStatuses
import app.indelible.offline.viewmodel.StorageViewModel
import app.indelible.onboarding.repository.ApiOnboardingRepository
import app.indelible.onboarding.repository.OnboardingRepository
import app.indelible.onboarding.viewmodel.OnboardingViewModel
import app.indelible.profile.repository.AccountRepository
import app.indelible.profile.repository.AddLibraryRepository
import app.indelible.profile.repository.ApiAccountRepository
import app.indelible.profile.repository.ApiAddLibraryRepository
import app.indelible.profile.repository.ApiMilaSettingsRepository
import app.indelible.profile.repository.ApiPreferencesRepository
import app.indelible.profile.repository.MilaSettingsRepository
import app.indelible.profile.repository.PreferencesRepository
import app.indelible.profile.viewmodel.AccountViewModel
import app.indelible.profile.viewmodel.AddLibraryViewModel
import app.indelible.profile.viewmodel.AiSettingsViewModel
import app.indelible.profile.viewmodel.UserPreferencesViewModel
import app.indelible.reader.repository.ApiReaderRepository
import app.indelible.reader.repository.ReaderOfflineCopy
import app.indelible.reader.repository.ReaderOutboxWrites
import app.indelible.reader.repository.ReaderRepository
import app.indelible.reader.repository.ReadingEventWriter
import app.indelible.search.repository.ApiSearchRepository
import app.indelible.search.repository.SearchRepository
import app.indelible.search.viewmodel.SearchViewModel
import app.indelible.share.SaveUrlUseCase
import app.indelible.share.repository.PendingSaveRepository
import app.indelible.sidebar.repository.ApiSidebarRepository
import app.indelible.sidebar.repository.SidebarRepository
import app.indelible.sidebar.viewmodel.SidebarViewModel
import app.indelible.tags.repository.ApiTagsRepository
import app.indelible.tags.repository.TagsRepository
import app.indelible.trash.repository.ApiTrashRepository
import app.indelible.trash.repository.TrashRepository
import io.ktor.util.date.getTimeMillis
import org.koin.core.module.Module
import org.koin.dsl.module

/** What the platform entry point hands the app: storage, the database and the device observers. */
data class AppDependencies(
    val tokenStorage: TokenStorage,
    val userPreferencesStorage: UserPreferencesStorage,
    val pendingSaveRepository: PendingSaveRepository,
    val databaseDriverFactory: DatabaseDriverFactory,
    val connectivityObserver: ConnectivityObserver,
    val offlineFilesRoot: OfflineFilesRoot,
)

internal fun appModule(
    deps: AppDependencies,
    oauthBrowserLauncher: OAuthBrowserLauncher,
    onUnauthorized: (epoch: Long) -> Unit,
): Module =
    module {
        single<TokenStorage> { deps.tokenStorage }
        single<UserPreferencesStorage> { deps.userPreferencesStorage }
        single<PendingSaveRepository> { deps.pendingSaveRepository }
        single { deps.connectivityObserver }
        single { OfflineDatabase(deps.databaseDriverFactory.createDriver()) }
        single { SessionRegistry() }
        single<OfflineStore> { SqlDelightOfflineStore(get(), registry = get()) }
        single {
            AuthenticatedApiTransport(
                tokenStorage = get(),
                onUnauthorized = onUnauthorized,
                registry = get(),
            )
        }
        single { ServerCapabilities(get(), clock = { getTimeMillis() }) }
        apiServices()
        single<AuthRepository> { ApiAuthRepository(get(), get()) }
        single { SaveUrlUseCase(get(), get(), get()) }
        single<HomeRepository> { ApiHomeRepository(get()) }
        single<LibraryRepository> { ApiLibraryRepository(get()) }
        single<FeedRepository> { ApiFeedRepository(get()) }
        readerRepositories()
        single { MilaRepository(get()) }
        single<OnboardingRepository> { ApiOnboardingRepository(get()) }
        single<SearchRepository> { ApiSearchRepository(get()) }
        single<SidebarRepository> { ApiSidebarRepository(get()) }
        single<CollectionsRepository> { ApiCollectionsRepository(get()) }
        single<TagsRepository> { ApiTagsRepository(get()) }
        single<TrashRepository> { ApiTrashRepository(get()) }
        single<AddLibraryRepository> { ApiAddLibraryRepository(get()) }
        single<AccountRepository> { ApiAccountRepository(get()) }
        single<MilaSettingsRepository> { ApiMilaSettingsRepository(get()) }
        single<PreferencesRepository> { ApiPreferencesRepository(get()) }
        single<ServerHealthChecker> { HttpServerHealthChecker() }
        single {
            ConnectServerViewModel(
                tokenStorage = get(),
                healthChecker = get(),
                sessions = get(),
                bakedDefaultUrl = ServerBuildConfig.SERVER_URL_DEFAULT,
                devPrefillUrl = ServerBuildConfig.DEV_SERVER_PREFILL,
            )
        }
        offlineServices(deps)
        single { OfflineAccount(get(), get(), get(), get(), deps.connectivityObserver.online) }
        single { AuthViewModel(get(), get(), get(), get(), oauthBrowserLauncher) }
        viewModels()
    }

private fun Module.apiServices() {
    single { LibraryApiService(get()) }
    single { FeedApiService(get()) }
    single { ReaderApiService(get()) }
    single { AuthApiService(get()) }
    single { AccountApiService(get()) }
    single { OnboardingApiService(get()) }
    single { CollectionsApiService(get()) }
    single { SearchApiService(get()) }
    single { TagsApiService(get()) }
    single { SettingsApiService(get()) }
    single { HomeApiService(get()) }
    single { MilaApiService(get()) }
    single { TrashApiService(get()) }
    single { ImportApiService(get()) }
}

private fun Module.readerRepositories() {
    single {
        val registry = get<SessionRegistry>()
        val downloads = get<DownloadManager>()
        ReaderOfflineCopy(
            store = get(),
            files = get(),
            sessionProvider = { registry.current.value.session },
            clock = { getTimeMillis() },
            copies = downloads,
        )
    }
    single {
        val registry = get<SessionRegistry>()
        ApiReaderRepository(
            readerApiService = get(),
            libraryApiService = get(),
            outbox = ReaderOutboxWrites(get(), get()),
            sessionProvider = { registry.current.value.session },
            offlineCopy = get(),
            capabilities = get(),
        )
    }
    single<ReaderRepository> { get<ApiReaderRepository>() }
    single<ReadingEventWriter> { get<ApiReaderRepository>() }
}

private fun Module.offlineServices(deps: AppDependencies) {
    single<OutboxSender> { ApiOutboxSender(get()) }
    single {
        OutboxWorker(
            store = get(),
            registry = get(),
            sender = get(),
            clock = { getTimeMillis() },
        )
    }
    single { SessionTransitions(get(), get(), get(), get()) }
    single { deps.offlineFilesRoot.offlineFiles() }
    single {
        val preferences = get<UserPreferencesStorage>()
        OfflineCopies(get(), get()) { preferences.getOfflineCapBytes() }
    }
    single {
        val preferences = get<UserPreferencesStorage>()
        OfflineSetFetcher(
            transport = get(),
            capabilities = get(),
            store = get(),
            files = get(),
            clock = { getTimeMillis() },
            capBytes = { preferences.getOfflineCapBytes() },
        )
    }
    single {
        DownloadManager(
            registry = get(),
            fetcher = get(),
            store = get(),
            copies = get(),
            online = deps.connectivityObserver.online,
        )
    }
    single {
        val downloads = get<DownloadManager>()
        val tags = get<ReaderOfflineCopy>().tags
        ScopePurger(get()) { scope ->
            tags.quiesce()
            downloads.removeAllDownloads(scope)
        }
    }
}

private fun Module.viewModels() {
    single { OnboardingViewModel(get(), get(), get()) }
    single { UserPreferencesViewModel(get(), get()) }
    single { OfflineStatuses(get(), get(), get<DownloadManager>().acquisitions) }
    single { LibraryViewModel(get(), get<OfflineStatuses>().observe()) }
    single { FeedViewModel(get()) }
    single { AddFeedViewModel(get()) }
    single { AddLibraryViewModel(get()) }
    single { FeedManagementViewModel(get()) }
    single { AccountViewModel(get()) }
    single { AiSettingsViewModel(get()) }
    single { SearchViewModel(get()) }
    single { SidebarViewModel(get()) }
    single {
        StorageViewModel(get(), get(), get(), get(), get<OutboxWorker>()::requestDrain)
    }
    single {
        val transport = get<AuthenticatedApiTransport>()
        val home = get<HomeRepository>()
        OfflineShelfViewModel(get(), get(), transport.reachable) { home.getDashboard() }
    }
}
