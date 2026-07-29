package app.indelible

import android.content.Intent
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.core.splashscreen.SplashScreen.Companion.installSplashScreen
import app.indelible.auth.oauth.OAuthCallbackBus
import app.indelible.core.storage.AndroidTokenStorage
import app.indelible.core.storage.AndroidUserPreferencesStorage

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        installSplashScreen()

        enableEdgeToEdge()
        super.onCreate(savedInstanceState)

        val tokenStorage = AndroidTokenStorage(applicationContext)
        val userPreferencesStorage = AndroidUserPreferencesStorage(applicationContext)
        setContent {
            App(
                tokenStorage = tokenStorage,
                userPreferencesStorage = userPreferencesStorage,
            )
        }
        handleOAuthIntent(intent)
    }

    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        setIntent(intent)
        handleOAuthIntent(intent)
    }

    private fun handleOAuthIntent(intent: Intent?) {
        intent
            ?.dataString
            ?.takeIf { it.startsWith("com.useindelible.app:/oauth/callback") }
            ?.let {
                OAuthCallbackBus.emit(it)
                intent.data = null
            }
    }
}
