package app.indelible

import android.content.Intent
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import app.indelible.core.network.AuthenticatedApiTransport
import app.indelible.core.network.LibraryApiService
import app.indelible.core.storage.AndroidTokenStorage
import app.indelible.share.SaveUrlUseCase
import app.indelible.share.repository.AndroidPendingSaveRepository
import app.indelible.share.ui.ShareBottomSheet
import app.indelible.share.viewmodel.ShareUiState
import app.indelible.share.viewmodel.ShareViewModel
import app.indelible.ui.theme.AppTheme
import kotlinx.coroutines.delay

class ShareReceiverActivity : ComponentActivity() {
    private var transport: AuthenticatedApiTransport? = null

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        val sharedUrl = extractUrl(intent)

        if (sharedUrl == null) {
            finish()
            return
        }

        val tokenStorage = AndroidTokenStorage(applicationContext)
        val apiTransport = AuthenticatedApiTransport(tokenStorage)
        transport = apiTransport
        val libraryApiService = LibraryApiService(apiTransport)
        val pendingSaveRepository = AndroidPendingSaveRepository(applicationContext)
        val saveUrlUseCase = SaveUrlUseCase(libraryApiService, tokenStorage, pendingSaveRepository)
        val viewModel = ShareViewModel(saveUrlUseCase)

        setContent {
            val uiState by viewModel.uiState.collectAsState()

            AppTheme(darkTheme = isSystemInDarkTheme()) {
                ShareBottomSheet(
                    url = sharedUrl,
                    uiState = uiState,
                    onSave = { viewModel.save(sharedUrl) },
                    onDismiss = { finish() },
                    onSignIn = { launchMainApp() },
                )

                LaunchedEffect(uiState) {
                    if (uiState is ShareUiState.Success ||
                        uiState is ShareUiState.AlreadySaved ||
                        uiState is ShareUiState.Queued
                    ) {
                        delay(DISMISS_DELAY_MS)
                        finish()
                    }
                }
            }
        }
    }

    override fun onDestroy() {
        transport?.close()
        transport = null
        super.onDestroy()
    }

    private fun extractUrl(intent: Intent): String? {
        if (intent.action != Intent.ACTION_SEND) return null
        if (intent.type != "text/plain") return null
        return intent.getStringExtra(Intent.EXTRA_TEXT)?.trim()
    }

    companion object {
        private const val DISMISS_DELAY_MS = 1200L
    }

    private fun launchMainApp() {
        val launchIntent = packageManager.getLaunchIntentForPackage(packageName)
        launchIntent?.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
        launchIntent?.let { startActivity(it) }
        finish()
    }
}
