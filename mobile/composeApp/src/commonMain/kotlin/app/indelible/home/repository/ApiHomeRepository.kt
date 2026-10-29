package app.indelible.home.repository

import app.indelible.api.generated.models.HomeItemResponse
import app.indelible.core.network.HomeApiService
import app.indelible.core.offline.ListSnapshots
import app.indelible.core.offline.SnapshotRow
import app.indelible.core.offline.offlineJson
import app.indelible.home.model.HomeDashboard

const val HOME_CONTINUE_KEY = "home:continue"
const val HOME_RECENT_KEY = "home:recent"

class ApiHomeRepository(
    private val homeApiService: HomeApiService,
    private val snapshots: ListSnapshots,
) : HomeRepository {
    override suspend fun getDashboard(): Result<HomeDashboard> {
        val session = snapshots.currentSession()
        return homeApiService.getHomeDashboard().onSuccess { dashboard ->
            // A widget the server left out has nothing to show, so its old rows go too.
            snapshots.keep(session, HOME_CONTINUE_KEY, dashboard.continueReading?.items.toSnapshotRows())
            snapshots.keep(session, HOME_RECENT_KEY, dashboard.recentlyAdded?.items.toSnapshotRows())
        }
    }
}

private fun List<HomeItemResponse>?.toSnapshotRows(): List<SnapshotRow> =
    orEmpty().map { item ->
        val json = offlineJson.encodeToString(HomeItemResponse.serializer(), item)
        SnapshotRow(item.id, item.id, item.itemType, item.title, json)
    }
