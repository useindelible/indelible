package app.indelible.core.network

import app.indelible.api.generated.client.ApiV1SettingsArchivalClient
import app.indelible.api.generated.client.ApiV1SettingsNotificationsClient
import app.indelible.api.generated.client.ApiV1SettingsPreferencesClient
import app.indelible.api.generated.models.ArchivalSettingsResponse
import app.indelible.api.generated.models.NotificationsSettingsResponse
import app.indelible.api.generated.models.PreferencesSettingsResponse

class SettingsApiService(
    private val transport: AuthenticatedApiTransport,
) {
    suspend fun getArchivalSettings(): Result<ArchivalSettingsResponse> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1SettingsArchivalClient(client).getArchival(configuration)
        }

    suspend fun updateArchivalSettings(body: ArchivalSettingsResponse): Result<ArchivalSettingsResponse> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1SettingsArchivalClient(client).updateArchival(body, configuration)
        }

    suspend fun getNotificationsSettings(): Result<NotificationsSettingsResponse> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1SettingsNotificationsClient(client).getNotifications(configuration)
        }

    suspend fun updateNotificationsSettings(body: NotificationsSettingsResponse): Result<NotificationsSettingsResponse> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1SettingsNotificationsClient(client).updateNotifications(body, configuration)
        }

    suspend fun getPreferences(): Result<PreferencesSettingsResponse> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1SettingsPreferencesClient(client).getPreferences(configuration)
        }

    suspend fun updatePreferences(body: PreferencesSettingsResponse): Result<PreferencesSettingsResponse> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1SettingsPreferencesClient(client).updatePreferences(body, configuration)
        }
}
