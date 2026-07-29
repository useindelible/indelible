package app.indelible.home.repository

import app.indelible.core.network.HomeApiService
import app.indelible.home.model.HomeDashboard

class ApiHomeRepository(
    private val homeApiService: HomeApiService,
) : HomeRepository {
    override suspend fun getDashboard(): Result<HomeDashboard> = homeApiService.getHomeDashboard()
}
