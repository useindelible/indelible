package app.indelible.navigation

import androidx.compose.runtime.remember
import androidx.navigation.NavGraphBuilder
import androidx.navigation.NavHostController
import androidx.navigation.NavType
import androidx.navigation.compose.composable
import androidx.navigation.navArgument
import androidx.savedstate.read
import app.indelible.mila.data.ChatScope
import app.indelible.mila.data.MilaRepository
import app.indelible.mila.ui.MilaChatScreen
import app.indelible.mila.viewmodel.MilaChatViewModel

/**
 * In-app Mila chat destinations (single-document, collection-scoped, and cross-item).
 * Extracted from [MainNavigation] to keep that file under the 600-line cap; the
 * three routes share only [navController] and [milaRepository].
 */
fun NavGraphBuilder.milaChatRoutes(
    navController: NavHostController,
    milaRepository: MilaRepository,
) {
    composable(
        route = MainRoutes.MILA_CHAT_ITEM,
        arguments =
            listOf(
                navArgument("displayTitle") {
                    type = NavType.StringType
                    nullable = true
                    defaultValue = null
                },
            ),
    ) { backStackEntry ->
        val itemId: String =
            backStackEntry.arguments
                ?.read { getStringOrNull("itemId") }
                ?: return@composable
        val displayTitle = backStackEntry.arguments?.read { getStringOrNull("displayTitle") }
        val chatScope = ChatScope.SingleDocument(itemId, displayTitle = displayTitle)
        val viewModel =
            remember(itemId) {
                MilaChatViewModel(milaRepository, chatScope)
            }
        MilaChatScreen(
            viewModel = viewModel,
            onBack = { navController.popBackStack() },
            onNavigateToAiSettings = {
                navController.navigate(MainRoutes.PROFILE_AI)
            },
            onNavigateToItem = { id ->
                navController.navigate(MainRoutes.reader(id))
            },
        )
    }
    composable(
        route = MainRoutes.MILA_CHAT_COLLECTION,
        arguments =
            listOf(
                navArgument("displayTitle") {
                    type = NavType.StringType
                    nullable = true
                    defaultValue = null
                },
            ),
    ) { backStackEntry ->
        val collectionId: String =
            backStackEntry.arguments
                ?.read { getStringOrNull("collectionId") }
                ?: return@composable
        val displayTitle = backStackEntry.arguments?.read { getStringOrNull("displayTitle") }
        val chatScope = ChatScope.Collection(collectionId, displayTitle = displayTitle)
        val viewModel =
            remember(collectionId) {
                MilaChatViewModel(milaRepository, chatScope)
            }
        MilaChatScreen(
            viewModel = viewModel,
            onBack = { navController.popBackStack() },
            onNavigateToAiSettings = {
                navController.navigate(MainRoutes.PROFILE_AI)
            },
            onNavigateToItem = { id ->
                navController.navigate(MainRoutes.reader(id))
            },
        )
    }
    composable(MainRoutes.MILA_CHAT_CROSS) {
        val chatScope = ChatScope.CrossItem
        val viewModel =
            remember {
                MilaChatViewModel(milaRepository, chatScope)
            }
        MilaChatScreen(
            viewModel = viewModel,
            onBack = { navController.popBackStack() },
            onNavigateToAiSettings = {
                navController.navigate(MainRoutes.PROFILE_AI)
            },
            onNavigateToItem = { id ->
                navController.navigate(MainRoutes.reader(id))
            },
        )
    }
}
