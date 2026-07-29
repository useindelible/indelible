package app.indelible.navigation

import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Icon
import androidx.compose.material3.NavigationBar
import androidx.compose.material3.NavigationBarItem
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.navigation.NavDestination.Companion.hierarchy
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.currentBackStackEntryAsState
import androidx.navigation.compose.rememberNavController
import app.indelible.auth.viewmodel.AuthState
import app.indelible.auth.viewmodel.AuthViewModel
import app.indelible.core.di.AppContainer
import app.indelible.core.preferences.DefaultViewPreference
import app.indelible.profile.viewmodel.UserPreferencesViewModel
import io.ktor.http.encodeURLQueryComponent

object MainRoutes {
    const val COLLECTIONS = "collections"
    const val COLLECTION_DETAIL = "collections/{collectionId}"
    const val TAGS = "tags"
    const val TAG_DETAIL = "tags/{tagId}"
    const val TRASH = "trash"

    fun collectionDetail(collectionId: String) = "collections/$collectionId"

    fun tagDetail(tagId: String) = "tags/$tagId"

    const val ITEM_DETAIL = "library/item/{itemId}"
    const val READER = "reader/{documentId}"
    const val MILA_CHAT_ITEM = "mila/chat/item/{itemId}?displayTitle={displayTitle}"
    const val MILA_CHAT_COLLECTION = "mila/chat/collection/{collectionId}?displayTitle={displayTitle}"
    const val MILA_CHAT_CROSS = "mila/chat/cross"
    const val PROFILE_EDIT = "profile/edit"
    const val PROFILE_PREFERENCES = "profile/preferences"
    const val PROFILE_AI = "profile/ai"
    const val PROFILE_INTEGRATIONS = "profile/integrations"
    const val PROFILE_ADD_LIBRARY = "profile/add-library"
    const val PROFILE_ADD_FEED = "profile/add-feed"
    const val PROFILE_CHANGE_PASSWORD = "profile/change-password"
    const val PROFILE_FEED_MANAGEMENT = "profile/feed-management"
    const val PROFILE_ACCOUNT = "profile/account"
    const val PROFILE_AI_PRESET_NEW = "profile/ai/presets/new"
    const val PROFILE_AI_PRESET_EDIT = "profile/ai/presets/{presetId}"

    fun aiPresetEdit(presetId: String) = "profile/ai/presets/$presetId"

    fun itemDetail(itemId: String): String = "library/item/$itemId"

    fun reader(documentId: String): String = "reader/$documentId"

    fun milaChatItem(
        itemId: String,
        displayTitle: String = "",
    ): String =
        if (displayTitle.isEmpty()) {
            "mila/chat/item/$itemId"
        } else {
            "mila/chat/item/$itemId?displayTitle=${displayTitle.encodeURLQueryComponent()}"
        }

    fun milaChatCollection(
        collectionId: String,
        displayTitle: String = "",
    ): String =
        if (displayTitle.isEmpty()) {
            "mila/chat/collection/$collectionId"
        } else {
            "mila/chat/collection/$collectionId?displayTitle=${displayTitle.encodeURLQueryComponent()}"
        }
}

@Composable
fun MainNavigation(
    authViewModel: AuthViewModel,
    appContainer: AppContainer,
    userPreferencesViewModel: UserPreferencesViewModel,
    modifier: Modifier = Modifier,
) {
    val navController = rememberNavController()
    val authState by authViewModel.authState.collectAsState()
    val ingestEmail = (authState as? AuthState.Authenticated)?.user?.ingestEmail
    val ingestLibraryEmail = (authState as? AuthState.Authenticated)?.user?.ingestLibraryEmail
    val navBackStackEntry by navController.currentBackStackEntryAsState()
    val currentDestination = navBackStackEntry?.destination

    val defaultViewPreference by userPreferencesViewModel.defaultView.collectAsState()
    val showBottomBar =
        TabItem.entries.any { tab ->
            currentDestination?.hierarchy?.any { it.route == tab.route } == true
        }

    // Drawer collections/smart-lists load once; the drawer is shared by Home, Library, and Feed,
    // so the fetch lives at the navigation root rather than inside a tab composable.
    LaunchedEffect(Unit) { appContainer.sidebarViewModel.load() }

    // Initial data loads live here, not in ViewModel init blocks: the container
    // builds every ViewModel before sign-in, so an init-time fetch fails with
    // "Session expired" and the tabs then render that stale error after login.
    // MainNavigation only composes once authenticated, and re-composes per login.
    LaunchedEffect(Unit) {
        appContainer.libraryViewModel.refresh()
        appContainer.feedViewModel.refresh()
        appContainer.searchViewModel.refresh()
        appContainer.aiSettingsViewModel.refresh()
        appContainer.feedManagementViewModel.loadSubscriptions()
    }

    // The shared sidebar drawer wraps the whole Scaffold so its scrim covers the bottom
    // navigation bar: opening the drawer takes over the full screen (matching the prototype)
    // instead of dimming only the content above the tab bar. Swipe-to-open is limited to the
    // tabs that expose a menu button.
    LibraryDrawerHost(
        authState = authState,
        authViewModel = authViewModel,
        libraryViewModel = appContainer.libraryViewModel,
        sidebarViewModel = appContainer.sidebarViewModel,
        currentRoute = currentDestination?.route,
        navController = navController,
        gesturesEnabled =
            currentDestination?.route in
                setOf(TabItem.HOME.route, TabItem.LIBRARY.route, TabItem.FEED.route),
    ) { openDrawer ->
        Scaffold(
            modifier = modifier,
            contentWindowInsets = WindowInsets(0, 0, 0, 0),
            bottomBar = {
                if (showBottomBar) {
                    NavigationBar {
                        // REVIEW remains a reachable route but is intentionally absent from the bar;
                        // the prototype tab set is Home, Library, Feed, Search, Profile.
                        TabItem.entries.filter { it != TabItem.REVIEW }.forEach { tab ->
                            val selected =
                                currentDestination?.hierarchy?.any {
                                    it.route == tab.route
                                } == true

                            NavigationBarItem(
                                selected = selected,
                                onClick = {
                                    navController.navigate(tab.route) {
                                        popUpTo(TabItem.HOME.route) {
                                            saveState = true
                                        }
                                        launchSingleTop = true
                                        restoreState = true
                                    }
                                },
                                icon = {
                                    Icon(
                                        imageVector = tab.icon,
                                        contentDescription = tab.label,
                                    )
                                },
                                label = { Text(tab.label) },
                            )
                        }
                    }
                }
            },
        ) { paddingValues ->
            LaunchedEffect(Unit) {
                // MainNavigation only composes once auth resolves to Authenticated,
                // so drive the preference sync from here. This guarantees a fresh
                // server fetch on every login (not just cold starts with a valid
                // session), so default-view routing reflects the server value.
                userPreferencesViewModel.refresh()
                // Home is the start destination and the effective default. Only Feed/Search
                // preferences redirect away from it; LIBRARY (which is also the unset default)
                // stays on Home, as the preference enum has no dedicated Home value to tell them apart.
                val targetTab =
                    when (userPreferencesViewModel.defaultView.value) {
                        DefaultViewPreference.FEED -> TabItem.FEED
                        DefaultViewPreference.SEARCH -> TabItem.SEARCH
                        DefaultViewPreference.LIBRARY -> null
                    }
                if (targetTab != null) {
                    navController.navigate(targetTab.route) {
                        popUpTo(TabItem.HOME.route) { inclusive = true }
                        launchSingleTop = true
                    }
                }
            }

            NavHost(
                navController = navController,
                startDestination = TabItem.HOME.route,
                modifier = Modifier.padding(bottom = paddingValues.calculateBottomPadding()),
            ) {
                tabRoutes(
                    navController = navController,
                    authViewModel = authViewModel,
                    authState = authState,
                    homeRepository = appContainer.homeRepository,
                    libraryViewModel = appContainer.libraryViewModel,
                    sidebarViewModel = appContainer.sidebarViewModel,
                    feedViewModel = appContainer.feedViewModel,
                    searchViewModel = appContainer.searchViewModel,
                    openDrawer = openDrawer,
                )
                contentRoutes(
                    navController = navController,
                    libraryRepository = appContainer.libraryRepository,
                    readerRepository = appContainer.readerRepository,
                    milaRepository = appContainer.milaRepository,
                    collectionsRepository = appContainer.collectionsRepository,
                    tagsRepository = appContainer.tagsRepository,
                    trashRepository = appContainer.trashRepository,
                )
                profileRoutes(
                    navController = navController,
                    authViewModel = authViewModel,
                    aiSettingsViewModel = appContainer.aiSettingsViewModel,
                    userPreferencesViewModel = userPreferencesViewModel,
                    addLibraryViewModel = appContainer.addLibraryViewModel,
                    addFeedViewModel = appContainer.addFeedViewModel,
                    feedManagementViewModel = appContainer.feedManagementViewModel,
                    accountViewModel = appContainer.accountViewModel,
                    accountRepository = appContainer.accountRepository,
                    milaSettingsRepository = appContainer.milaSettingsRepository,
                    ingestEmail = ingestEmail,
                    ingestLibraryEmail = ingestLibraryEmail,
                )
                milaChatRoutes(navController, appContainer.milaRepository)
            }
        }
    }
}
