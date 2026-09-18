package app.indelible.core.storage

import app.indelible.core.preferences.DefaultViewPreference
import app.indelible.core.preferences.ThemePreference

/** How much the offline copies of one account may use before unpinned ones are evicted. */
const val DEFAULT_OFFLINE_CAP_BYTES = 1024L * 1024 * 1024

interface UserPreferencesStorage {
    suspend fun saveTheme(theme: ThemePreference)

    suspend fun getTheme(): ThemePreference

    suspend fun saveDefaultView(view: DefaultViewPreference)

    suspend fun getDefaultView(): DefaultViewPreference

    suspend fun saveOfflineCapBytes(bytes: Long)

    suspend fun getOfflineCapBytes(): Long
}
