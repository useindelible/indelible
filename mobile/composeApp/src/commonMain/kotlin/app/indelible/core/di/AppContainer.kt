package app.indelible.core.di

import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
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
import app.indelible.core.storage.TokenStorage
import app.indelible.core.storage.UserPreferencesStorage
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
import app.indelible.search.repository.ApiSearchRepository
import app.indelible.search.repository.SearchRepository
import app.indelible.search.viewmodel.SearchViewModel
import app.indelible.sidebar.repository.ApiSidebarRepository
import app.indelible.sidebar.repository.SidebarRepository
import app.indelible.sidebar.viewmodel.SidebarViewModel
import app.indelible.tags.repository.ApiTagsRepository
import app.indelible.tags.repository.TagsRepository
import app.indelible.trash.repository.ApiTrashRepository
import app.indelible.trash.repository.TrashRepository
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
)

@Composable
fun rememberAppContainer(
    tokenStorage: TokenStorage,
    userPreferencesStorage: UserPreferencesStorage,
): AppContainer {
    val oauthBrowserLauncher = rememberOAuthBrowserLauncher()
    val authViewModelRef = remember { mutableStateOf<AuthViewModel?>(null) }
    val koinApplication =
        remember(tokenStorage, userPreferencesStorage, oauthBrowserLauncher) {
            koinApplication {
                modules(
                    module {
                        single<TokenStorage> { tokenStorage }
                        single<UserPreferencesStorage> { userPreferencesStorage }
                        single {
                            AuthenticatedApiTransport(
                                tokenStorage = get(),
                                onUnauthorized = {
                                    authViewModelRef.value?.forceLogout()
                                },
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
                        single<HomeRepository> { ApiHomeRepository(get()) }
                        single<LibraryRepository> { ApiLibraryRepository(get()) }
                        single<FeedRepository> { ApiFeedRepository(get()) }
                        single<ReaderRepository> { ApiReaderRepository(get(), get()) }
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
                                bakedDefaultUrl = ServerBuildConfig.SERVER_URL_DEFAULT,
                                devPrefillUrl = ServerBuildConfig.DEV_SERVER_PREFILL,
                            )
                        }
                        single { AuthViewModel(get(), get(), oauthBrowserLauncher) }
                        single { OnboardingViewModel(get()) }
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
    val authViewModel = remember(koin) { koin.get<AuthViewModel>() }

    DisposableEffect(koinApplication) {
        onDispose {
            koin.get<AuthenticatedApiTransport>().close()
            koinApplication.close()
        }
    }

    LaunchedEffect(authViewModel) {
        authViewModelRef.value = authViewModel
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
        )
    }
}
