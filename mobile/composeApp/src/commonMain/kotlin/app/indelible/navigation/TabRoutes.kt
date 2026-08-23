package app.indelible.navigation

import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.navigation.NavGraphBuilder
import androidx.navigation.NavHostController
import androidx.navigation.compose.composable
import app.indelible.auth.viewmodel.AuthState
import app.indelible.auth.viewmodel.AuthViewModel
import app.indelible.feed.ui.FeedScreen
import app.indelible.feed.viewmodel.FeedViewModel
import app.indelible.home.repository.HomeRepository
import app.indelible.home.ui.HomeScreen
import app.indelible.home.viewmodel.HomeViewModel
import app.indelible.library.ui.LibraryScreen
import app.indelible.library.viewmodel.LibraryViewModel
import app.indelible.profile.ui.ProfileTab
import app.indelible.profile.viewmodel.AddLibraryViewModel
import app.indelible.search.ui.SearchScreen
import app.indelible.search.viewmodel.SearchViewModel
import app.indelible.sidebar.viewmodel.SidebarUiState
import app.indelible.sidebar.viewmodel.SidebarViewModel
import app.indelible.stubs.TabStub
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.nav_daily_review
import kotlinx.datetime.Clock
import kotlinx.datetime.TimeZone
import kotlinx.datetime.toLocalDateTime
import org.jetbrains.compose.resources.stringResource

fun NavGraphBuilder.tabRoutes(
    navController: NavHostController,
    authViewModel: AuthViewModel,
    authState: app.indelible.auth.viewmodel.AuthState,
    homeRepository: HomeRepository,
    libraryViewModel: LibraryViewModel,
    addLibraryViewModel: AddLibraryViewModel,
    sidebarViewModel: SidebarViewModel,
    feedViewModel: FeedViewModel,
    searchViewModel: SearchViewModel,
    openDrawer: () -> Unit,
) {
    composable(TabItem.HOME.route) {
        val homeViewModel =
            remember {
                HomeViewModel(
                    homeRepository,
                    nowHour = {
                        Clock.System
                            .now()
                            .toLocalDateTime(TimeZone.currentSystemDefault())
                            .hour
                    },
                )
            }
        LaunchedEffect(Unit) { homeViewModel.load() }
        val authUser = (authState as? AuthState.Authenticated)?.user
        val avatarBytes by authViewModel.avatarBytes.collectAsState()
        HomeScreen(
            viewModel = homeViewModel,
            userDisplayName = authUser?.displayName?.takeIf { it.isNotBlank() } ?: authUser?.email,
            onMenuClick = openDrawer,
            onSearchClick = {
                navController.navigate(TabItem.SEARCH.route) {
                    popUpTo(TabItem.HOME.route) { saveState = true }
                    launchSingleTop = true
                    restoreState = true
                }
            },
            onProfileClick = {
                navController.navigate(TabItem.PROFILE.route) {
                    popUpTo(TabItem.HOME.route) { saveState = true }
                    launchSingleTop = true
                    restoreState = true
                }
            },
            onOpenItem = { itemId ->
                navController.navigate(MainRoutes.reader(itemId))
            },
            avatarUrl = authUser?.avatarUrl,
            avatarBytes = avatarBytes,
        )
    }
    composable(TabItem.LIBRARY.route) {
        val authUser = (authState as? AuthState.Authenticated)?.user
        val avatarBytes by authViewModel.avatarBytes.collectAsState()
        val sidebarReady = sidebarViewModel.uiState.collectAsState().value as? SidebarUiState.Ready
        LibraryScreen(
            viewModel = libraryViewModel,
            addLibraryViewModel = addLibraryViewModel,
            onNavigateToItem = { itemId ->
                navController.navigate(MainRoutes.reader(itemId))
            },
            onMenuClick = openDrawer,
            onProfileClick = {
                navController.navigate(TabItem.PROFILE.route) {
                    popUpTo(TabItem.HOME.route) { saveState = true }
                    launchSingleTop = true
                    restoreState = true
                }
            },
            collections = sidebarReady?.collections ?: emptyList(),
            smartLists = sidebarReady?.smartLists ?: emptyList(),
            userDisplayName = authUser?.displayName?.takeIf { it.isNotBlank() } ?: authUser?.email,
            avatarUrl = authUser?.avatarUrl,
            avatarBytes = avatarBytes,
        )
    }
    composable(TabItem.FEED.route) {
        val authUser = (authState as? AuthState.Authenticated)?.user
        val avatarBytes by authViewModel.avatarBytes.collectAsState()
        FeedScreen(
            viewModel = feedViewModel,
            onNavigateToAddFeed = {
                navController.navigate(MainRoutes.PROFILE_ADD_FEED)
            },
            onNavigateToReader = { documentId ->
                navController.navigate(MainRoutes.reader(documentId))
            },
            onMenuClick = openDrawer,
            onProfileClick = {
                navController.navigate(TabItem.PROFILE.route) {
                    popUpTo(TabItem.HOME.route) { saveState = true }
                    launchSingleTop = true
                    restoreState = true
                }
            },
            onManageSources = {
                navController.navigate(MainRoutes.PROFILE_FEED_MANAGEMENT)
            },
            userDisplayName = authUser?.displayName?.takeIf { it.isNotBlank() } ?: authUser?.email,
            avatarUrl = authUser?.avatarUrl,
            avatarBytes = avatarBytes,
        )
    }
    composable(TabItem.SEARCH.route) {
        SearchScreen(
            viewModel = searchViewModel,
            onNavigateToReader = { itemId ->
                navController.navigate(MainRoutes.reader(itemId))
            },
        )
    }
    composable(TabItem.REVIEW.route) { TabStub(title = stringResource(Res.string.nav_daily_review)) }
    composable(TabItem.PROFILE.route) {
        ProfileTab(
            authViewModel = authViewModel,
            onNavigateToEdit = { navController.navigate(MainRoutes.PROFILE_EDIT) },
            onNavigateToPreferences = { navController.navigate(MainRoutes.PROFILE_PREFERENCES) },
            onNavigateToAi = { navController.navigate(MainRoutes.PROFILE_AI) },
            onNavigateToIntegrations = { navController.navigate(MainRoutes.PROFILE_INTEGRATIONS) },
            onNavigateToAccount = { navController.navigate(MainRoutes.PROFILE_ACCOUNT) },
        )
    }
}
