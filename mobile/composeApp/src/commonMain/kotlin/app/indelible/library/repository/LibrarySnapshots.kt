package app.indelible.library.repository

import app.indelible.api.generated.models.LibraryEntryResponse
import app.indelible.core.model.PageInfo
import app.indelible.core.model.PaginatedItems
import app.indelible.core.model.toLibraryItem
import app.indelible.core.offline.SnapshotRow
import app.indelible.core.offline.offlineJson

private const val ALL = "all"
private const val TRIAGE = "triage:"

internal fun triageKey(
    state: String?,
    type: String?,
): String = "$TRIAGE${state ?: ALL}:${type ?: ALL}"

internal fun collectionKey(id: String): String = "collection:$id"

internal fun smartListKey(id: String): String = "smartlist:$id"

/** An entry now in [newState] leaves only the triage lists of another state; every other list waits for a fetch. */
internal fun keepAfterTriage(newState: String): (String) -> Boolean =
    { key -> !key.startsWith(TRIAGE) || key.startsWith("$TRIAGE$ALL:") || key.startsWith("$TRIAGE$newState:") }

internal fun LibraryEntryResponse.toSnapshotRow(): SnapshotRow =
    SnapshotRow(
        itemId = libraryEntryId,
        documentId = documentId,
        documentType = documentType,
        title = title,
        rowJson = offlineJson.encodeToString(LibraryEntryResponse.serializer(), this),
    )

internal fun List<SnapshotRow>.toDevicePage(): PaginatedItems? {
    val items =
        mapNotNull { row ->
            runCatching { offlineJson.decodeFromString(LibraryEntryResponse.serializer(), row.rowJson) }
                .getOrNull()
                ?.toLibraryItem()
        }
    if (items.isEmpty()) return null
    return PaginatedItems(items, PageInfo(hasMore = false, nextCursor = null), fromDevice = true)
}
