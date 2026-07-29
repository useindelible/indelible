package app.indelible.home.repository

import app.indelible.home.model.HomeDashboard

interface HomeRepository {
    suspend fun getDashboard(): Result<HomeDashboard>
}
