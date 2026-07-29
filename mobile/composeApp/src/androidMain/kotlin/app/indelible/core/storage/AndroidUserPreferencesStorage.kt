package app.indelible.core.storage

import android.content.Context
import android.content.SharedPreferences
import app.indelible.core.preferences.DefaultViewPreference
import app.indelible.core.preferences.ThemePreference

class AndroidUserPreferencesStorage(
    context: Context,
) : UserPreferencesStorage {
    private val prefs: SharedPreferences =
        context.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)

    override suspend fun saveTheme(theme: ThemePreference) {
        prefs.edit().putString(KEY_THEME, theme.name).commit()
    }

    override suspend fun getTheme(): ThemePreference {
        val name = prefs.getString(KEY_THEME, null) ?: return ThemePreference.AUTO
        return ThemePreference.entries.firstOrNull { it.name == name } ?: ThemePreference.AUTO
    }

    override suspend fun saveDefaultView(view: DefaultViewPreference) {
        prefs.edit().putString(KEY_DEFAULT_VIEW, view.name).commit()
    }

    override suspend fun getDefaultView(): DefaultViewPreference {
        val name = prefs.getString(KEY_DEFAULT_VIEW, null) ?: return DefaultViewPreference.LIBRARY
        return DefaultViewPreference.entries.firstOrNull { it.name == name } ?: DefaultViewPreference.LIBRARY
    }

    companion object {
        private const val PREFS_NAME = "indelible_preferences"
        private const val KEY_THEME = "theme"
        private const val KEY_DEFAULT_VIEW = "default_view"
    }
}
