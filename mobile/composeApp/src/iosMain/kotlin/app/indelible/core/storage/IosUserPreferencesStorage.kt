package app.indelible.core.storage

import app.indelible.core.preferences.DefaultViewPreference
import app.indelible.core.preferences.ThemePreference
import platform.Foundation.NSUserDefaults

class IosUserPreferencesStorage : UserPreferencesStorage {
    private val defaults = NSUserDefaults.standardUserDefaults

    override suspend fun saveTheme(theme: ThemePreference) {
        defaults.setObject(theme.name, KEY_THEME)
    }

    override suspend fun getTheme(): ThemePreference {
        val name = defaults.stringForKey(KEY_THEME) ?: return ThemePreference.AUTO
        return ThemePreference.entries.firstOrNull { it.name == name } ?: ThemePreference.AUTO
    }

    override suspend fun saveDefaultView(view: DefaultViewPreference) {
        defaults.setObject(view.name, KEY_DEFAULT_VIEW)
    }

    override suspend fun getDefaultView(): DefaultViewPreference {
        val name = defaults.stringForKey(KEY_DEFAULT_VIEW) ?: return DefaultViewPreference.LIBRARY
        return DefaultViewPreference.entries.firstOrNull { it.name == name } ?: DefaultViewPreference.LIBRARY
    }

    companion object {
        private const val KEY_THEME = "pref_theme"
        private const val KEY_DEFAULT_VIEW = "pref_default_view"
    }
}
