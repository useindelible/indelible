package app.indelible.home.viewmodel

import app.indelible.api.generated.models.ContinueReadingWidget
import app.indelible.api.generated.models.ReadingStatsWidget
import app.indelible.api.generated.models.RecentlyAddedWidget
import app.indelible.home.model.HomeDashboard
import app.indelible.home.model.HomeItem
import app.indelible.home.repository.HomeRepository
import kotlinx.datetime.Instant

class FakeHomeRepository(
    var dashboardResult: Result<HomeDashboard> = Result.success(emptyDashboard()),
) : HomeRepository {
    var getDashboardCallCount = 0

    override suspend fun getDashboard(): Result<HomeDashboard> {
        getDashboardCallCount++
        return dashboardResult
    }

    companion object {
        private val fixedInstant = Instant.parse("2026-01-01T00:00:00Z")

        fun item(
            id: String = "lib_1",
            title: String = "The Untold Story",
            itemType: String = "article",
            domain: String? = "theatlantic.com",
            progressPercent: Float? = 62f,
            readingTimeMinutes: Int? = 8,
        ) = HomeItem(
            id = id,
            title = title,
            itemType = itemType,
            createdAt = fixedInstant,
            domain = domain,
            progressPercent = progressPercent,
            readingTimeMinutes = readingTimeMinutes,
        )

        fun emptyDashboard() = HomeDashboard()

        fun sampleDashboard() =
            HomeDashboard(
                continueReading =
                    ContinueReadingWidget(
                        items = listOf(item("lib_1"), item("lib_2", "Second Read")),
                    ),
                readingStats =
                    ReadingStatsWidget(
                        documentsRead = 7,
                        highlightsMade = 14,
                        itemsCompleted = 9,
                        streakDays = 12,
                    ),
                recentlyAdded =
                    RecentlyAddedWidget(
                        items = listOf(item("lib_3", "Fresh Save")),
                    ),
            )
    }
}
