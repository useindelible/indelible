package app.indelible.core.di

import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import app.indelible.auth.oauth.rememberOAuthBrowserLauncher
import app.indelible.auth.repository.ApiAuthRepository
import app.indelible.auth.repository.AuthRepository
import app.indelible.auth.server.HttpServerHealthChecker
import app.indelible.auth.server.ServerHealthChecker
import app.indelible.auth.viewmodel.AuthViewModel
import app.indelible.auth.viewmodel.ConnectServerViewModel
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
import app.indelible.library.repository.ApiLibraryRepository
import app.indelible.library.repository.LibraryRepository
import app.indelible.library.viewmodel.LibraryViewModel
import app.indelible.mila.data.MilaRepository
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
import org.koin.dsl.koinApplication
import org.koin.dsl.module

data class AppContainer(
    val apiTransport: AuthenticatedApiTransport,
    val authViewModel: AuthViewModel,
    val connectServerViewModel: ConnectServerViewModel,
    val onboardingViewModel: OnboardingViewModel,
    val userPreferencesViewModel: UserPreferencesViewModel,
    val homeRepository: HomeRepository,
    val libraryRepository: LibraryRepository,
    val feedRepository: FeedRepository,
    val readerRepository: ReaderRepository,
    val readingEventWriter: ReadingEventWriter,
    val milaRepository: MilaRepository,
    val searchRepository: SearchRepository,
    val sidebarRepository: SidebarRepository,
    val collectionsRepository: CollectionsRepository,
    val tagsRepository: TagsRepository,
    val trashRepository: TrashRepository,
    val accountRepository: AccountRepository,
    val milaSettingsRepository: MilaSettingsRepository,
    val libraryViewModel: LibraryViewModel,
    val feedViewModel: FeedViewModel,
    val addFeedViewModel: AddFeedViewModel,
    val addLibraryViewModel: AddLibraryViewModel,
    val feedManagementViewModel: FeedManagementViewModel,
    val accountViewModel: AccountViewModel,
    val aiSettingsViewModel: AiSettingsViewModel,
    val searchViewModel: SearchViewModel,
    val sidebarViewModel: SidebarViewModel,
    val outboxWorker: OutboxWorker,
    val sessionTransitions: SessionTransitions,
    val scopePurger: ScopePurger,
    val connectivityObserver: ConnectivityObserver,
    val downloads: DownloadManager,
)

@Composable
fun rememberAppContainer(
    tokenStorage: TokenStorage,
    userPreferencesStorage: UserPreferencesStorage,
    pendingSaveRepository: PendingSaveRepository,
    databaseDriverFactory: DatabaseDriverFactory,
    connectivityObserver: ConnectivityObserver,
    offlineFilesRoot: OfflineFilesRoot,
): AppContainer {
    val oauthBrowserLauncher = rememberOAuthBrowserLauncher()
    val authViewModelRef = remember { mutableStateOf<AuthViewModel?>(null) }
    val koinApplication =
        remember(tokenStorage, userPreferencesStorage, oauthBrowserLauncher, databaseDriverFactory, offlineFilesRoot) {
            koinApplication {
                modules(
                    module {
                        single<TokenStorage> { tokenStorage }
                        single<UserPreferencesStorage> { userPreferencesStorage }
                        single<PendingSaveRepository> { pendingSaveRepository }
                        single { connectivityObserver }
                        single { OfflineDatabase(databaseDriverFactory.createDriver()) }
                        single { SessionRegistry() }
                        single<OfflineStore> { SqlDelightOfflineStore(get(), registry = get()) }
                        single {
                            AuthenticatedApiTransport(
                                tokenStorage = get(),
                                onUnauthorized = { epoch ->
                                    authViewModelRef.value?.forceLogout(epoch)
                                },
                                registry = get(),
                            )
                        }
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
                        single<AuthRepository> { ApiAuthRepository(get(), get()) }
                        single { SaveUrlUseCase(get(), get(), get()) }
                        single<HomeRepository> { ApiHomeRepository(get()) }
                        single<LibraryRepository> { ApiLibraryRepository(get()) }
                        single<FeedRepository> { ApiFeedRepository(get()) }
                        single {
                            val registry = get<SessionRegistry>()
                            ApiReaderRepository(
                                readerApiService = get(),
                                libraryApiService = get(),
                                offlineStore = get(),
                                worker = get(),
                                sessionProvider = { registry.current.value.session },
                            )
                        }
                        single<ReaderRepository> { get<ApiReaderRepository>() }
                        single<ReadingEventWriter> { get<ApiReaderRepository>() }
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
                        single { offlineFilesRoot.offlineFiles() }
                        single {
                            val preferences = get<UserPreferencesStorage>()
                            OfflineCopies(get(), get()) { preferences.getOfflineCapBytes() }
                        }
                        single {
                            val preferences = get<UserPreferencesStorage>()
                            OfflineSetFetcher(
                                transport = get(),
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
                                online = connectivityObserver.online,
                            )
                        }
                        single {
                            val downloads = get<DownloadManager>()
                            ScopePurger(get()) { scope -> downloads.removeAllDownloads(scope) }
                        }
                        single { AuthViewModel(get(), get(), get(), get(), get(), oauthBrowserLauncher) }
                        single { OnboardingViewModel(get(), get(), get()) }
                        single { UserPreferencesViewModel(get(), get()) }
                        single { LibraryViewModel(get()) }
                        single { FeedViewModel(get()) }
                        single { AddFeedViewModel(get()) }
                        single { AddLibraryViewModel(get()) }
                        single { FeedManagementViewModel(get()) }
                        single { AccountViewModel(get()) }
                        single { AiSettingsViewModel(get()) }
                        single { SearchViewModel(get()) }
                        single { SidebarViewModel(get()) }
                    },
                )
            }
        }
    val koin = koinApplication.koin
    // Assigned during the first composition, not from an effect: a 401 resolved before the first
    // frame's effects ran would otherwise withdraw the session without ever signing the user out.
    val authViewModel = remember(koin) { koin.get<AuthViewModel>().also { authViewModelRef.value = it } }

    DisposableEffect(koinApplication) {
        koin.get<OutboxWorker>().start()
        koin.get<DownloadManager>().start()
        onDispose {
            koin.get<DownloadManager>().stop()
            koin.get<OutboxWorker>().stop()
            koin.get<AuthenticatedApiTransport>().close()
            koinApplication.close()
        }
    }

    return remember(koin, authViewModel) {
        AppContainer(
            apiTransport = koin.get(),
            authViewModel = authViewModel,
            connectServerViewModel = koin.get(),
            onboardingViewModel = koin.get(),
            userPreferencesViewModel = koin.get(),
            homeRepository = koin.get(),
            libraryRepository = koin.get(),
            feedRepository = koin.get(),
            readerRepository = koin.get(),
            readingEventWriter = koin.get(),
            milaRepository = koin.get(),
            searchRepository = koin.get(),
            sidebarRepository = koin.get(),
            collectionsRepository = koin.get(),
            tagsRepository = koin.get(),
            trashRepository = koin.get(),
            accountRepository = koin.get(),
            milaSettingsRepository = koin.get(),
            libraryViewModel = koin.get(),
            feedViewModel = koin.get(),
            addFeedViewModel = koin.get(),
            addLibraryViewModel = koin.get(),
            feedManagementViewModel = koin.get(),
            accountViewModel = koin.get(),
            aiSettingsViewModel = koin.get(),
            searchViewModel = koin.get(),
            sidebarViewModel = koin.get(),
            outboxWorker = koin.get(),
            sessionTransitions = koin.get(),
            scopePurger = koin.get(),
            connectivityObserver = connectivityObserver,
            downloads = koin.get(),
        )
    }
}
