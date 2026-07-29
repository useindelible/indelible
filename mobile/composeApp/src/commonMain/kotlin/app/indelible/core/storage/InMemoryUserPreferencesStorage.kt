package app.indelible.core.storage

import app.indelible.core.preferences.DefaultViewPreference
import app.indelible.core.preferences.ThemePreference

class InMemoryUserPreferencesStorage : UserPreferencesStorage {
    private var theme = ThemePreference.AUTO
    private var defaultView = DefaultViewPreference.LIBRARY

    override suspend fun saveTheme(theme: ThemePreference) {
        this.theme = theme
    }

    override suspend fun getTheme(): ThemePreference = theme

    override suspend fun saveDefaultView(view: DefaultViewPreference) {
        this.defaultView = view
    }

    override suspend fun getDefaultView(): DefaultViewPreference = defaultView
}
