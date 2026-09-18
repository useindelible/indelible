package app.indelible.core.offline

import app.indelible.api.generated.models.HighlightWithNoteResponse
import app.indelible.api.generated.models.LocatorSchemaFlat
import kotlinx.serialization.json.Json

private val locatorJson = Json { ignoreUnknownKeys = true }

/** The server's highlight in the cached shape the local view replays queued changes over. */
fun HighlightWithNoteResponse.toCachedHighlight(documentId: String): CachedHighlight =
    CachedHighlight(
        id = id,
        documentId = documentId,
        color = color,
        textContent = textContent,
        locator = locator?.let { locatorJson.encodeToString(LocatorSchemaFlat.serializer(), it) },
        tags = tags,
        note = note?.let { CachedHighlightNote(it.body) },
        createdAtEpochMs = createdAt.toEpochMilliseconds(),
        updatedAtEpochMs = updatedAt.toEpochMilliseconds(),
    )
