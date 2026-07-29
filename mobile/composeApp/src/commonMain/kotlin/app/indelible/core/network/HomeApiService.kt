package app.indelible.core.network

import app.indelible.api.generated.client.ApiV1HomeClient
import app.indelible.api.generated.client.ApiV1SettingsHomeClient
import app.indelible.api.generated.models.HomeDashboardResponse
import app.indelible.api.generated.models.HomeSettingsResponse
import app.indelible.api.generated.models.UpdateHomeSettingsBody

class HomeApiService(
    private val transport: AuthenticatedApiTransport,
) {
    suspend fun getHomeDashboard(): Result<HomeDashboardResponse> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1HomeClient(client).getHome(apiConfiguration = configuration)
        }

    suspend fun getHomeSettings(): Result<HomeSettingsResponse> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1SettingsHomeClient(client).getHomeSettings(configuration)
        }

    suspend fun updateHomeSettings(body: UpdateHomeSettingsBody): Result<HomeSettingsResponse> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1SettingsHomeClient(client).updateHomeSettings(body, configuration)
        }
}
