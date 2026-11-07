@file:OptIn(kotlinx.cinterop.ExperimentalForeignApi::class)

package app.indelible

import androidx.compose.runtime.remember
import androidx.compose.ui.window.ComposeUIViewController
import app.indelible.core.i18n.AppLanguage
import app.indelible.core.i18n.AppLanguageSettings
import app.indelible.core.offline.ConnectivityObserver
import app.indelible.core.storage.IosTokenStorage
import app.indelible.core.storage.IosUserPreferencesStorage
import app.indelible.db.DatabaseDriverFactory
import app.indelible.share.repository.IosPendingSaveRepository
import platform.Foundation.NSBundle
import platform.Foundation.NSURL
import platform.UIKit.UIApplication
import platform.UIKit.UIApplicationOpenSettingsURLString

@Suppress("ktlint:standard:function-naming", "FunctionNaming")
fun MainViewController() =
    ComposeUIViewController {
        val databaseDriverFactory = remember { DatabaseDriverFactory() }
        val connectivityObserver = remember { ConnectivityObserver() }
        val appLanguageSettings =
            AppLanguageSettings.SystemManaged(
                language =
                    AppLanguage.fromLanguageTag(
                        NSBundle.mainBundle.preferredLocalizations.firstOrNull() as? String ?: "en",
                    ),
                onOpenSettings = {
                    NSURL.URLWithString(UIApplicationOpenSettingsURLString)?.let { url ->
                        UIApplication.sharedApplication.openURL(
                            url = url,
                            options = emptyMap<Any?, Any>(),
                            completionHandler = null,
                        )
                    }
                },
            )
        App(
            tokenStorage = IosTokenStorage(),
            userPreferencesStorage = IosUserPreferencesStorage(),
            pendingSaveRepository = IosPendingSaveRepository(),
            databaseDriverFactory = databaseDriverFactory,
            connectivityObserver = connectivityObserver,
            appLanguageSettings = appLanguageSettings,
        )
    }
