package app.indelible.core.model

import app.indelible.api.generated.models.LibraryScopeCountsResponse

/**
 * Read-state and per-item-type totals for one library scope, backing the header
 * clearance meter and the content-type chip counts.
 *
 * [byItemType] is keyed by the API's item-type string (article, video, pdf, ...) and
 * omits types with nothing saved, so callers must treat a missing key as zero.
 */
data class LibraryCounts(
    val total: Int,
    val unread: Int,
    val reading: Int,
    val done: Int,
    val byItemType: Map<String, Int>,
) {
    companion object {
        val EMPTY = LibraryCounts(total = 0, unread = 0, reading = 0, done = 0, byItemType = emptyMap())
    }
}

fun LibraryScopeCountsResponse.toLibraryCounts(): LibraryCounts =
    LibraryCounts(
        total = total.toInt(),
        unread = unread.toInt(),
        reading = reading.toInt(),
        done = done.toInt(),
        byItemType = byItemType.associate { it.itemType to it.count.toInt() },
    )
