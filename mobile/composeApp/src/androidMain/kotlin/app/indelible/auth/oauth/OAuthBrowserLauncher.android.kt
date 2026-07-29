package app.indelible.auth.oauth

import android.content.Context
import android.net.Uri
import androidx.browser.customtabs.CustomTabsIntent
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.platform.LocalContext

class AndroidOAuthBrowserLauncher(
    private val context: Context,
) : OAuthBrowserLauncher {
    override suspend fun launch(url: String): Result<Unit> =
        runCatching {
            CustomTabsIntent
                .Builder()
                .build()
                .launchUrl(context, Uri.parse(url))
        }
}

@Composable
actual fun rememberOAuthBrowserLauncher(): OAuthBrowserLauncher {
    val context = LocalContext.current
    return remember(context) { AndroidOAuthBrowserLauncher(context) }
}
