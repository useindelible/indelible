package app.indelible

import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.Surface
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import app.indelible.auth.navigation.AuthNavigation
import app.indelible.auth.viewmodel.AuthState
import app.indelible.core.di.rememberAppContainer
import app.indelible.core.i18n.AppLanguageSettings
import app.indelible.core.image.newImageLoader
import app.indelible.core.preferences.ThemePreference
import app.indelible.core.storage.TokenStorage
import app.indelible.core.storage.UserPreferencesStorage
import app.indelible.navigation.MainNavigation
import app.indelible.onboarding.ui.OnboardingFlow
import app.indelible.share.repository.PendingSaveRepository
import app.indelible.ui.components.AppStartupSplash
import app.indelible.ui.components.StartupSplashGate
import app.indelible.ui.theme.AppTheme
import coil3.compose.setSingletonImageLoaderFactory

@Composable
fun App(
    tokenStorage: TokenStorage,
    userPreferencesStorage: UserPreferencesStorage,
    pendingSaveRepository: PendingSaveRepository,
    appLanguageSettings: AppLanguageSettings? = null,
    onReady: () -> Unit = {},
) {
    val appContainer = rememberAppContainer(tokenStorage, userPreferencesStorage, pendingSaveRepository)
    val transport = appContainer.apiTransport
    setSingletonImageLoaderFactory { context -> newImageLoader(context, transport) }
    val authViewModel = appContainer.authViewModel
    val userPreferencesViewModel = appContainer.userPreferencesViewModel
    val authState by authViewModel.authState.collectAsState()
    val themePreference by userPreferencesViewModel.theme.collectAsState()
    val systemDark = isSystemInDarkTheme()
    val darkTheme =
        when (themePreference) {
            ThemePreference.LIGHT -> false
            ThemePreference.DARK -> true
            ThemePreference.AUTO -> systemDark
        }

    AppTheme(darkTheme = darkTheme) {
        Surface(modifier = Modifier.fillMaxSize()) {
            StartupSplashGate(
                isReady = authState !is AuthState.Loading,
                onNativeSplashReady = onReady,
            ) {
                when (authState) {
                    is AuthState.Loading -> {
                        AppStartupSplash()
                    }
                    is AuthState.Unauthenticated -> {
                        AuthNavigation(
                            viewModel = authViewModel,
                            connectServerViewModel = appContainer.connectServerViewModel,
                        )
                    }
                    is AuthState.NeedsVerification -> {
                        AuthNavigation(
                            viewModel = authViewModel,
                            connectServerViewModel = appContainer.connectServerViewModel,
                        )
                    }
                    is AuthState.NeedsOnboarding -> {
                        val onboardingViewModel = appContainer.onboardingViewModel
                        OnboardingFlow(
                            viewModel = onboardingViewModel,
                            onComplete = { authViewModel.initialize() },
                        )
                    }
                    is AuthState.Authenticated -> {
                        MainNavigation(
                            authViewModel = authViewModel,
                            appContainer = appContainer,
                            userPreferencesViewModel = userPreferencesViewModel,
                            appLanguageSettings = appLanguageSettings,
                        )
                    }
                }
            }
        }
    }
}
