package app.indelible.profile.repository

import app.indelible.api.generated.models.PreferencesSettingsResponse

interface PreferencesRepository {
    suspend fun getPreferences(): Result<PreferencesSettingsResponse>

    suspend fun updatePreferences(body: PreferencesSettingsResponse): Result<PreferencesSettingsResponse>
}
