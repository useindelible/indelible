package app.indelible.navigation

import androidx.compose.runtime.remember
import androidx.navigation.NavGraphBuilder
import androidx.navigation.NavHostController
import androidx.navigation.compose.composable
import androidx.savedstate.read
import app.indelible.collections.repository.CollectionsRepository
import app.indelible.collections.ui.CollectionDetailScreen
import app.indelible.collections.ui.CollectionsScreen
import app.indelible.collections.viewmodel.CollectionDetailViewModel
import app.indelible.collections.viewmodel.CollectionsViewModel
import app.indelible.library.repository.LibraryRepository
import app.indelible.library.ui.ItemDetailScreen
import app.indelible.library.viewmodel.ItemDetailViewModel
import app.indelible.mila.data.ChatScope
import app.indelible.mila.data.MilaRepository
import app.indelible.mila.viewmodel.MilaChatViewModel
import app.indelible.reader.repository.ReaderRepository
import app.indelible.reader.ui.ReaderScreen
import app.indelible.reader.viewmodel.ReaderViewModel
import app.indelible.tags.repository.TagsRepository
import app.indelible.tags.ui.TagDetailScreen
import app.indelible.tags.ui.TagsScreen
import app.indelible.tags.viewmodel.TagDetailViewModel
import app.indelible.tags.viewmodel.TagsViewModel
import app.indelible.trash.repository.TrashRepository
import app.indelible.trash.ui.TrashScreen
import app.indelible.trash.viewmodel.TrashViewModel

fun NavGraphBuilder.contentRoutes(
    navController: NavHostController,
    libraryRepository: LibraryRepository,
    readerRepository: ReaderRepository,
    milaRepository: MilaRepository,
    collectionsRepository: CollectionsRepository,
    tagsRepository: TagsRepository,
    trashRepository: TrashRepository,
) {
    composable(MainRoutes.ITEM_DETAIL) { backStackEntry ->
        val itemId: String =
            backStackEntry.arguments
                ?.read { getStringOrNull("itemId") }
                ?: return@composable
        val itemDetailViewModel =
            remember(itemId) {
                ItemDetailViewModel(itemId, libraryRepository)
            }
        ItemDetailScreen(
            viewModel = itemDetailViewModel,
            onNavigateBack = { navController.popBackStack() },
            onOpenInReader = { id ->
                navController.navigate(MainRoutes.reader(id))
            },
        )
    }
    composable(MainRoutes.READER) { backStackEntry ->
        val documentId: String =
            backStackEntry.arguments
                ?.read { getStringOrNull("documentId") }
                ?: return@composable
        val readerViewModel =
            remember(documentId) {
                ReaderViewModel(documentId, readerRepository)
            }
        ReaderScreen(
            viewModel = readerViewModel,
            onNavigateBack = { navController.popBackStack() },
            milaViewModelProvider = { title ->
                MilaChatViewModel(
                    milaRepository,
                    ChatScope.SingleDocument(documentId, displayTitle = title),
                )
            },
            onNavigateToAiSettings = {
                navController.navigate(MainRoutes.PROFILE_AI)
            },
            onNavigateToItem = { id ->
                navController.navigate(MainRoutes.reader(id))
            },
        )
    }
    composable(MainRoutes.COLLECTIONS) {
        val collectionsViewModel = remember { CollectionsViewModel(collectionsRepository) }
        CollectionsScreen(
            viewModel = collectionsViewModel,
            onNavigateBack = { navController.popBackStack() },
            onNavigateToCollection = { id ->
                navController.navigate(MainRoutes.collectionDetail(id))
            },
        )
    }
    composable(MainRoutes.COLLECTION_DETAIL) { backStackEntry ->
        val collectionId: String =
            backStackEntry.arguments
                ?.read { getStringOrNull("collectionId") }
                ?: return@composable
        val detailViewModel =
            remember(collectionId) {
                CollectionDetailViewModel(collectionId, collectionsRepository)
            }
        CollectionDetailScreen(
            viewModel = detailViewModel,
            onNavigateBack = { navController.popBackStack() },
            onNavigateToCollection = { id ->
                navController.navigate(MainRoutes.collectionDetail(id))
            },
            onNavigateToItem = { id ->
                navController.navigate(MainRoutes.reader(id))
            },
        )
    }
    composable(MainRoutes.TAGS) {
        val tagsViewModel = remember { TagsViewModel(tagsRepository) }
        TagsScreen(
            viewModel = tagsViewModel,
            onNavigateBack = { navController.popBackStack() },
            onNavigateToTag = { tagId -> navController.navigate(MainRoutes.tagDetail(tagId)) },
        )
    }
    composable(MainRoutes.TAG_DETAIL) { backStackEntry ->
        val tagId: String =
            backStackEntry.arguments
                ?.read { getStringOrNull("tagId") }
                ?: return@composable
        val tagDetailViewModel =
            remember(tagId) {
                TagDetailViewModel(tagId, tagsRepository)
            }
        TagDetailScreen(
            viewModel = tagDetailViewModel,
            onNavigateBack = { navController.popBackStack() },
            onNavigateToTag = { id -> navController.navigate(MainRoutes.tagDetail(id)) },
            onNavigateToItem = { id -> navController.navigate(MainRoutes.reader(id)) },
        )
    }
    composable(MainRoutes.TRASH) {
        val trashViewModel = remember { TrashViewModel(trashRepository) }
        TrashScreen(
            viewModel = trashViewModel,
            onNavigateBack = { navController.popBackStack() },
        )
    }
}
