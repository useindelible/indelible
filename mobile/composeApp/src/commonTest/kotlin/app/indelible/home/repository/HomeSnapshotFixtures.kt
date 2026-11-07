package app.indelible.home.repository

import app.indelible.core.offline.SnapshotRow

private const val WHEN = "2026-01-01T00:00:00Z"

internal fun homeItemJson(
    id: String,
    type: String = "pdf",
    domain: String? = null,
    progress: Float? = null,
): String =
    """{"id":"$id","item_type":"$type","title":"Title $id","created_at":"$WHEN",""" +
        """"domain":${domain?.let { "\"$it\"" } ?: "null"},"progress_percent":${progress ?: "null"}}"""

internal fun dashboardJson(
    continueIds: List<String>?,
    recentIds: List<String>?,
): String {
    fun widget(ids: List<String>?) = ids?.let { """{"items":[${it.joinToString(",") { id -> homeItemJson(id) }}]}""" }
    val fields =
        listOfNotNull(
            widget(continueIds)?.let { "\"continue_reading\":$it" },
            widget(recentIds)?.let { "\"recently_added\":$it" },
        )
    return fields.joinToString(",", prefix = "{", postfix = "}")
}

internal fun homeRow(
    id: String,
    type: String = "pdf",
    domain: String? = null,
    progress: Float? = null,
): SnapshotRow = SnapshotRow(id, id, type, "Title $id", homeItemJson(id, type, domain, progress))
