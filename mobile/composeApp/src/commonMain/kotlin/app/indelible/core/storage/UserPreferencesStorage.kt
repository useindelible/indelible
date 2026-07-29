package app.indelible.core.storage

import app.indelible.core.preferences.DefaultViewPreference
import app.indelible.core.preferences.ThemePreference

interface UserPreferencesStorage {
    suspend fun saveTheme(theme: ThemePreference)

    suspend fun getTheme(): ThemePreference

    suspend fun saveDefaultView(view: DefaultViewPreference)

    suspend fun getDefaultView(): DefaultViewPreference
}
