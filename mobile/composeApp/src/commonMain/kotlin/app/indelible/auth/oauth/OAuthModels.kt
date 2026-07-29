package app.indelible.auth.oauth

import androidx.compose.runtime.Composable
import kotlinx.serialization.Serializable

const val NativeOAuthRedirectUri = "com.useindelible.app:/oauth/callback"

@Serializable
data class PendingOAuthFlow(
    val providerId: String,
    val verifier: String,
    val appState: String,
    val serverUrl: String,
    val expiresAtEpochSeconds: Long,
)

data class OAuthProviderUi(
    val id: String,
    val name: String,
)

data class OAuthCallbackResult(
    val code: String?,
    val state: String?,
    val error: String?,
    val errorDescription: String?,
)

interface OAuthBrowserLauncher {
    suspend fun launch(url: String): Result<Unit>
}

object NoopOAuthBrowserLauncher : OAuthBrowserLauncher {
    override suspend fun launch(url: String): Result<Unit> =
        Result.failure(
            IllegalStateException("OAuth browser launcher is unavailable"),
        )
}

@Composable
expect fun rememberOAuthBrowserLauncher(): OAuthBrowserLauncher
