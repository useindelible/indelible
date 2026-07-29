package app.indelible.navigation

import androidx.compose.foundation.layout.widthIn
import androidx.compose.material3.DrawerValue
import androidx.compose.material3.ModalDrawerSheet
import androidx.compose.material3.ModalNavigationDrawer
import androidx.compose.material3.rememberDrawerState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.navigation.NavHostController
import app.indelible.auth.viewmodel.AuthState
import app.indelible.auth.viewmodel.AuthViewModel
import app.indelible.library.viewmodel.ContentTypeFilter
import app.indelible.library.viewmodel.LibraryScope
import app.indelible.library.viewmodel.LibraryViewModel
import app.indelible.sidebar.ui.LibrarySidebarSheet
import app.indelible.sidebar.viewmodel.SidebarUiState
import app.indelible.sidebar.viewmodel.SidebarViewModel
import kotlinx.coroutines.launch

/**
 * Hosts the shared navigation drawer (profile, content-type filters, collections,
 * smart lists, settings/trash) and renders [content] in front of it. This wraps the
 * whole app shell so the drawer's scrim covers the bottom navigation bar — opening it
 * takes over the full screen, matching the prototype. [content] receives an `openDrawer`
 * lambda to bind to its menu button; [gesturesEnabled] limits swipe-to-open to the tabs
 * that surface that button (an already-open drawer can always be swiped closed).
 */
@Composable
internal fun LibraryDrawerHost(
    authState: AuthState,
    authViewModel: AuthViewModel,
    libraryViewModel: LibraryViewModel,
    sidebarViewModel: SidebarViewModel,
    currentRoute: String?,
    navController: NavHostController,
    gesturesEnabled: Boolean = true,
    content: @Composable (openDrawer: () -> Unit) -> Unit,
) {
    val drawerState = rememberDrawerState(DrawerValue.Closed)
    val drawerScope = rememberCoroutineScope()
    val authUser = (authState as? AuthState.Authenticated)?.user
    val email = authUser?.email.orEmpty()
    val realName = authUser?.displayName?.takeIf { it.isNotBlank() }
    val displayName = realName ?: email
    // Subtitle only carries the email when the header title is a real name; otherwise the
    // title already falls back to the email and a duplicate line would be noise.
    val sidebarSubtitle = if (realName != null) email else ""
    val avatarBytes by authViewModel.avatarBytes.collectAsState()
    val contentTypeFilter by libraryViewModel.contentTypeFilter.collectAsState()
    val sidebarState by sidebarViewModel.uiState.collectAsState()
    val sidebarReady = sidebarState as? SidebarUiState.Ready
    val collections = sidebarReady?.collections ?: emptyList()
    val smartLists = sidebarReady?.smartLists ?: emptyList()

    ModalNavigationDrawer(
        drawerState = drawerState,
        gesturesEnabled = gesturesEnabled || drawerState.isOpen,
        drawerContent = {
            ModalDrawerSheet(
                modifier = Modifier.widthIn(max = 312.dp),
            ) {
                LibrarySidebarSheet(
                    displayName = displayName,
                    currentRoute = currentRoute,
                    currentContentType = contentTypeFilter.apiValue,
                    collections = collections,
                    smartLists = smartLists,
                    onNavigateToContentType = { apiValue ->
                        drawerScope.launch { drawerState.close() }
                        libraryViewModel.setContentTypeFilter(ContentTypeFilter.fromApiValue(apiValue))
                        navController.navigate(TabItem.LIBRARY.route) {
                            popUpTo(TabItem.HOME.route) { saveState = true }
                            launchSingleTop = true
                            restoreState = true
                        }
                    },
                    onNavigateToCollection = { id ->
                        drawerScope.launch { drawerState.close() }
                        navController.navigate(MainRoutes.collectionDetail(id))
                    },
                    onNavigateToSmartList = { id ->
                        drawerScope.launch { drawerState.close() }
                        val name = smartLists.find { it.id == id }?.name.orEmpty()
                        libraryViewModel.setScope(LibraryScope.SmartList(id, name))
                        navController.navigate(TabItem.LIBRARY.route) {
                            popUpTo(TabItem.HOME.route) { saveState = true }
                            launchSingleTop = true
                            restoreState = true
                        }
                    },
                    onNewCollection = {
                        drawerScope.launch { drawerState.close() }
                        navController.navigate(MainRoutes.COLLECTIONS)
                    },
                    onNewSmartList = {
                        // No smart-list create screen exists yet; Phase 3 wires it alongside
                        // the scope switcher. Dismiss the drawer for now.
                        drawerScope.launch { drawerState.close() }
                    },
                    onNavigateToSettings = {
                        drawerScope.launch { drawerState.close() }
                        navController.navigate(TabItem.PROFILE.route) {
                            popUpTo(TabItem.HOME.route) { saveState = true }
                            launchSingleTop = true
                            restoreState = true
                        }
                    },
                    onNavigateToTrash = {
                        drawerScope.launch { drawerState.close() }
                        navController.navigate(MainRoutes.TRASH)
                    },
                    subtitle = sidebarSubtitle,
                    avatarUrl = authUser?.avatarUrl,
                    avatarBytes = avatarBytes,
                )
            }
        },
    ) {
        content { drawerScope.launch { drawerState.open() } }
    }
}
