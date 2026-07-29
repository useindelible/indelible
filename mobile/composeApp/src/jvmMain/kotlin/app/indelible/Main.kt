package app.indelible

import androidx.compose.runtime.remember
import androidx.compose.ui.window.Window
import androidx.compose.ui.window.application
import app.indelible.core.storage.InMemoryTokenStorage
import app.indelible.core.storage.JvmTokenStorage
import app.indelible.core.storage.JvmUserPreferencesStorage
import app.indelible.core.storage.pendingQueueOwner
import app.indelible.share.repository.JvmPendingSaveRepository

fun main() =
    application {
        val tokenState = remember { InMemoryTokenStorage() }
        val userPreferencesStorage = remember { JvmUserPreferencesStorage() }
        val pendingSaveRepository = remember(tokenState) {
            JvmPendingSaveRepository { tokenState.pendingQueueOwner() }
        }
        val tokenStorage = remember(tokenState, pendingSaveRepository) {
            JvmTokenStorage(tokenState, pendingSaveRepository)
        }

        Window(
            onCloseRequest = ::exitApplication,
            title = "Indelible",
        ) {
            App(
                tokenStorage = tokenStorage,
                userPreferencesStorage = userPreferencesStorage,
                pendingSaveRepository = pendingSaveRepository,
            )
        }
    }
