package app.indelible

import android.app.LocaleManager
import android.content.Intent
import android.os.Build
import android.os.Bundle
import android.os.LocaleList
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.core.splashscreen.SplashScreen.Companion.installSplashScreen
import app.indelible.auth.oauth.OAuthCallbackBus
import app.indelible.core.i18n.AppLanguage
import app.indelible.core.i18n.AppLanguageSettings
import app.indelible.core.storage.AndroidTokenStorage
import app.indelible.core.storage.AndroidUserPreferencesStorage
import app.indelible.share.repository.AndroidPendingSaveRepository

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        installSplashScreen()

        enableEdgeToEdge()
        super.onCreate(savedInstanceState)

        val tokenStorage = AndroidTokenStorage(applicationContext)
        val userPreferencesStorage = AndroidUserPreferencesStorage(applicationContext)
        val pendingSaveRepository = AndroidPendingSaveRepository(applicationContext)
        val appLanguageSettings =
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
                val localeManager = getSystemService(LocaleManager::class.java)
                val applicationLocales = localeManager.applicationLocales
                AppLanguageSettings.Selectable(
                    language =
                        if (applicationLocales.isEmpty) {
                            AppLanguage.SYSTEM_DEFAULT
                        } else {
                            AppLanguage.fromLanguageTag(applicationLocales[0].toLanguageTag())
                        },
                    onSelected = { language ->
                        localeManager.applicationLocales =
                            language.languageTag?.let(LocaleList::forLanguageTags)
                                ?: LocaleList.getEmptyLocaleList()
                    },
                )
            } else {
                null
            }
        setContent {
            App(
                tokenStorage = tokenStorage,
                userPreferencesStorage = userPreferencesStorage,
                pendingSaveRepository = pendingSaveRepository,
                appLanguageSettings = appLanguageSettings,
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
