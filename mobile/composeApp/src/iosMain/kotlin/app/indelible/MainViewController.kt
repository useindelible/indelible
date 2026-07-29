package app.indelible

import androidx.compose.ui.window.ComposeUIViewController
import app.indelible.core.storage.IosTokenStorage
import app.indelible.core.storage.IosUserPreferencesStorage
import app.indelible.share.repository.IosPendingSaveRepository

@Suppress("ktlint:standard:function-naming", "FunctionNaming")
fun MainViewController() =
    ComposeUIViewController {
        App(
            tokenStorage = IosTokenStorage(),
            userPreferencesStorage = IosUserPreferencesStorage(),
            pendingSaveRepository = IosPendingSaveRepository(),
        )
    }
