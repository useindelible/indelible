package app.indelible.core.di

import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import app.indelible.auth.oauth.rememberOAuthBrowserLauncher
import app.indelible.auth.viewmodel.AuthViewModel
import app.indelible.auth.viewmodel.ConnectServerViewModel
import app.indelible.collections.repository.CollectionsRepository
import app.indelible.core.network.AuthenticatedApiTransport
import app.indelible.core.offline.ConnectivityObserver
import app.indelible.core.offline.DownloadManager
import app.indelible.core.offline.OfflineFilesRoot
import app.indelible.core.offline.OutboxWorker
import app.indelible.core.offline.ScopePurger
import app.indelible.core.offline.SessionTransitions
import app.indelible.core.storage.TokenStorage
import app.indelible.core.storage.UserPreferencesStorage
import app.indelible.db.DatabaseDriverFactory
import app.indelible.feed.repository.FeedRepository
import app.indelible.feed.viewmodel.AddFeedViewModel
import app.indelible.feed.viewmodel.FeedManagementViewModel
import app.indelible.feed.viewmodel.FeedViewModel
import app.indelible.home.repository.HomeRepository
import app.indelible.home.viewmodel.OfflineShelfViewModel
import app.indelible.library.repository.LibraryRepository
import app.indelible.library.viewmodel.LibraryViewModel
import app.indelible.mila.data.MilaRepository
import app.indelible.offline.viewmodel.DocumentOfflineViewModel
import app.indelible.offline.viewmodel.StorageViewModel
import app.indelible.onboarding.viewmodel.OnboardingViewModel
import app.indelible.profile.repository.AccountRepository
import app.indelible.profile.repository.MilaSettingsRepository
import app.indelible.profile.viewmodel.AccountViewModel
import app.indelible.profile.viewmodel.AddLibraryViewModel
import app.indelible.profile.viewmodel.AiSettingsViewModel
import app.indelible.profile.viewmodel.UserPreferencesViewModel
import app.indelible.reader.repository.ReaderRepository
import app.indelible.reader.repository.ReadingEventWriter
import app.indelible.search.repository.SearchRepository
import app.indelible.search.viewmodel.SearchViewModel
import app.indelible.share.repository.PendingSaveRepository
import app.indelible.sidebar.repository.SidebarRepository
import app.indelible.sidebar.viewmodel.SidebarViewModel
import app.indelible.tags.repository.TagsRepository
import app.indelible.trash.repository.TrashRepository
import org.koin.core.Koin
import org.koin.dsl.koinApplication

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
    val storageViewModel: StorageViewModel,
    val offlineShelf: OfflineShelfViewModel,
    val documentOffline: (String) -> DocumentOfflineViewModel,
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
            val deps =
                AppDependencies(
                    tokenStorage,
                    userPreferencesStorage,
                    pendingSaveRepository,
                    databaseDriverFactory,
                    connectivityObserver,
                    offlineFilesRoot,
                )
            koinApplication {
                modules(appModule(deps, oauthBrowserLauncher) { epoch -> authViewModelRef.value?.forceLogout(epoch) })
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

    return remember(koin, authViewModel) { appContainer(koin, authViewModel, connectivityObserver) }
}

private fun appContainer(
    koin: Koin,
    authViewModel: AuthViewModel,
    connectivityObserver: ConnectivityObserver,
): AppContainer =
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
        storageViewModel = koin.get(),
        offlineShelf = koin.get(),
        documentOffline = { id -> DocumentOfflineViewModel(id, koin.get(), koin.get(), koin.get()) },
    )
