package app.indelible.profile.repository

import app.indelible.api.generated.models.PreferencesSettingsResponse
import app.indelible.core.network.SettingsApiService

class ApiPreferencesRepository(
    private val settingsApiService: SettingsApiService,
) : PreferencesRepository {
    override suspend fun getPreferences(): Result<PreferencesSettingsResponse> = settingsApiService.getPreferences()

    override suspend fun updatePreferences(body: PreferencesSettingsResponse): Result<PreferencesSettingsResponse> =
        settingsApiService.updatePreferences(body)
}
