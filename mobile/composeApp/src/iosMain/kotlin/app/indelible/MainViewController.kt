package app.indelible

import androidx.compose.ui.window.ComposeUIViewController
import app.indelible.core.storage.IosTokenStorage
import app.indelible.core.storage.IosUserPreferencesStorage

@Suppress("ktlint:standard:function-naming", "FunctionNaming")
fun MainViewController() =
    ComposeUIViewController {
        App(
            tokenStorage = IosTokenStorage(),
            userPreferencesStorage = IosUserPreferencesStorage(),
        )
    }
