package app.indelible.navigation

import androidx.compose.runtime.remember
import androidx.navigation.NavGraphBuilder
import androidx.navigation.NavHostController
import androidx.navigation.compose.composable
import androidx.savedstate.read
import app.indelible.auth.viewmodel.AuthViewModel
import app.indelible.feed.ui.AddFeedScreen
import app.indelible.feed.ui.FeedManagementScreen
import app.indelible.feed.viewmodel.AddFeedViewModel
import app.indelible.feed.viewmodel.FeedManagementViewModel
import app.indelible.profile.repository.AccountRepository
import app.indelible.profile.repository.MilaSettingsRepository
import app.indelible.profile.ui.AccountScreen
import app.indelible.profile.ui.AddLibraryScreen
import app.indelible.profile.ui.AiSettingsScreen
import app.indelible.profile.ui.ChangePasswordScreen
import app.indelible.profile.ui.IntegrationsScreen
import app.indelible.profile.ui.PreferencesScreen
import app.indelible.profile.ui.ProfileEditScreen
import app.indelible.profile.ui.PromptPresetEditScreen
import app.indelible.profile.viewmodel.AccountViewModel
import app.indelible.profile.viewmodel.AddLibraryViewModel
import app.indelible.profile.viewmodel.AiSettingsViewModel
import app.indelible.profile.viewmodel.ChangePasswordViewModel
import app.indelible.profile.viewmodel.PromptPresetEditViewModel
import app.indelible.profile.viewmodel.UserPreferencesViewModel

fun NavGraphBuilder.profileRoutes(
    navController: NavHostController,
    authViewModel: AuthViewModel,
    aiSettingsViewModel: AiSettingsViewModel,
    userPreferencesViewModel: UserPreferencesViewModel,
    addLibraryViewModel: AddLibraryViewModel,
    addFeedViewModel: AddFeedViewModel,
    feedManagementViewModel: FeedManagementViewModel,
    accountViewModel: AccountViewModel,
    accountRepository: AccountRepository,
    milaSettingsRepository: MilaSettingsRepository,
    ingestEmail: String?,
    ingestLibraryEmail: String?,
) {
    composable(MainRoutes.PROFILE_EDIT) {
        ProfileEditScreen(
            authViewModel = authViewModel,
            onNavigateBack = { navController.popBackStack() },
        )
    }
    composable(MainRoutes.PROFILE_PREFERENCES) {
        PreferencesScreen(
            viewModel = userPreferencesViewModel,
            onNavigateBack = { navController.popBackStack() },
        )
    }
    composable(MainRoutes.PROFILE_AI) {
        AiSettingsScreen(
            viewModel = aiSettingsViewModel,
            onNavigateBack = { navController.popBackStack() },
            onNavigateToPreset = { presetId ->
                if (presetId == null) {
                    navController.navigate(MainRoutes.PROFILE_AI_PRESET_NEW)
                } else {
                    navController.navigate(MainRoutes.aiPresetEdit(presetId))
                }
            },
        )
    }
    composable(MainRoutes.PROFILE_AI_PRESET_NEW) {
        val vm =
            remember {
                PromptPresetEditViewModel(repository = milaSettingsRepository, existingPreset = null)
            }
        PromptPresetEditScreen(
            viewModel = vm,
            onNavigateBack = {
                aiSettingsViewModel.reloadPresets()
                navController.popBackStack()
            },
        )
    }
    composable(MainRoutes.PROFILE_AI_PRESET_EDIT) { backStackEntry ->
        val presetId =
            backStackEntry.arguments?.read { getStringOrNull("presetId") }
                ?: return@composable
        val existing =
            aiSettingsViewModel.uiState.value.presets
                .find { it.id == presetId }
        val vm =
            remember(presetId) {
                PromptPresetEditViewModel(repository = milaSettingsRepository, existingPreset = existing)
            }
        PromptPresetEditScreen(
            viewModel = vm,
            onNavigateBack = {
                aiSettingsViewModel.reloadPresets()
                navController.popBackStack()
            },
        )
    }
    composable(MainRoutes.PROFILE_INTEGRATIONS) {
        IntegrationsScreen(
            ingestEmail = ingestEmail,
            ingestLibraryEmail = ingestLibraryEmail,
            onNavigateBack = { navController.popBackStack() },
            onNavigateToAddLibrary = { navController.navigate(MainRoutes.PROFILE_ADD_LIBRARY) },
            onNavigateToAddFeed = { navController.navigate(MainRoutes.PROFILE_ADD_FEED) },
            onNavigateToFeeds = { navController.navigate(MainRoutes.PROFILE_FEED_MANAGEMENT) },
        )
    }
    composable(MainRoutes.PROFILE_ADD_LIBRARY) {
        AddLibraryScreen(
            viewModel = addLibraryViewModel,
            ingestLibraryEmail = ingestLibraryEmail,
            onNavigateBack = { navController.popBackStack() },
        )
    }
    composable(MainRoutes.PROFILE_ADD_FEED) {
        AddFeedScreen(
            viewModel = addFeedViewModel,
            ingestEmail = ingestEmail,
            onNavigateBack = { navController.popBackStack() },
        )
    }
    composable(MainRoutes.PROFILE_FEED_MANAGEMENT) {
        FeedManagementScreen(
            viewModel = feedManagementViewModel,
            onNavigateBack = { navController.popBackStack() },
            onNavigateToAddFeed = {
                navController.navigate(MainRoutes.PROFILE_ADD_FEED)
            },
        )
    }
    composable(MainRoutes.PROFILE_ACCOUNT) {
        AccountScreen(
            viewModel = accountViewModel,
            onNavigateBack = { navController.popBackStack() },
            onNavigateToChangePassword = { navController.navigate(MainRoutes.PROFILE_CHANGE_PASSWORD) },
            onSignOut = { authViewModel.logout() },
            onAccountDeleted = { authViewModel.forceLogout() },
        )
    }
    composable(MainRoutes.PROFILE_CHANGE_PASSWORD) {
        val vm = remember { ChangePasswordViewModel(accountRepository) }
        ChangePasswordScreen(
            viewModel = vm,
            onNavigateBack = { navController.popBackStack() },
        )
    }
}
