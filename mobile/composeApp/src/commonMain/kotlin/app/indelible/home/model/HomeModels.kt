package app.indelible.home.model

typealias HomeDashboard = app.indelible.api.generated.models.HomeDashboardResponse
typealias HomeItem = app.indelible.api.generated.models.HomeItemResponse
typealias ReadingStatsWidget = app.indelible.api.generated.models.ReadingStatsWidget

private const val PERCENT_MAX = 100f

internal val HomeItem.progressFraction: Float
    get() = ((progressPercent ?: 0f) / PERCENT_MAX).coerceIn(0f, 1f)
