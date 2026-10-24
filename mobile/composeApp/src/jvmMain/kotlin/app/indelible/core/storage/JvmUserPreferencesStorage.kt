package app.indelible.core.storage

import app.indelible.core.preferences.DefaultViewPreference
import app.indelible.core.preferences.ThemePreference
import java.util.prefs.Preferences

class JvmUserPreferencesStorage : UserPreferencesStorage {
    private val prefs = Preferences.userNodeForPackage(JvmUserPreferencesStorage::class.java)

    override suspend fun saveTheme(theme: ThemePreference) {
        prefs.put(KEY_THEME, theme.name)
        prefs.flush()
    }

    override suspend fun getTheme(): ThemePreference {
        val name = prefs.get(KEY_THEME, null) ?: return ThemePreference.AUTO
        return ThemePreference.entries.firstOrNull { it.name == name } ?: ThemePreference.AUTO
    }

    override suspend fun saveDefaultView(view: DefaultViewPreference) {
        prefs.put(KEY_DEFAULT_VIEW, view.name)
        prefs.flush()
    }

    override suspend fun getDefaultView(): DefaultViewPreference {
        val name = prefs.get(KEY_DEFAULT_VIEW, null) ?: return DefaultViewPreference.LIBRARY
        return DefaultViewPreference.entries.firstOrNull { it.name == name } ?: DefaultViewPreference.LIBRARY
    }

    override suspend fun saveOfflineCapBytes(bytes: Long) {
        prefs.putLong(KEY_OFFLINE_CAP_BYTES, bytes)
        prefs.flush()
    }

    override suspend fun getOfflineCapBytes(): Long = prefs.getLong(KEY_OFFLINE_CAP_BYTES, DEFAULT_OFFLINE_CAP_BYTES)

    companion object {
        private const val KEY_THEME = "pref_theme"
        private const val KEY_DEFAULT_VIEW = "pref_default_view"
        private const val KEY_OFFLINE_CAP_BYTES = "pref_offline_cap_bytes"
    }
}
