package app.indelible

import androidx.compose.ui.window.Window
import androidx.compose.ui.window.application
import app.indelible.core.storage.InMemoryTokenStorage
import app.indelible.core.storage.JvmUserPreferencesStorage

fun main() =
    application {
        Window(
            onCloseRequest = ::exitApplication,
            title = "Indelible",
        ) {
            App(
                tokenStorage = InMemoryTokenStorage(),
                userPreferencesStorage = JvmUserPreferencesStorage(),
            )
        }
    }
