package app.indelible.reader.repository

import app.indelible.core.offline.LocalChanges
import app.indelible.core.offline.Progress
import app.indelible.core.offline.SnapshotRow
import app.indelible.core.offline.offlineJson
import app.indelible.reader.model.ReaderDocument
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.contentOrNull
import kotlinx.serialization.json.doubleOrNull
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive

// An article would poll, then say it is still being prepared; these open on a status screen with Retry.
private val OPENS_ON_STATUS = setOf("pdf", "book")

/** Enough of an unfetchable item to pick its PDF or book screen; a progress still [queued] here wins. */
internal fun SnapshotRow.toLastSeenDocument(queued: LocalChanges): ReaderDocument? {
    if (documentType !in OPENS_ON_STATUS) return null
    val fields = runCatching { offlineJson.parseToJsonElement(rowJson).jsonObject }.getOrNull()
    val triage = fields?.text("triage_state")
    val document = ReaderDocument(
        id = documentId,
        itemType = documentType,
        title = title,
        saved = true,
        readableReady = false,
        availableAssets = emptyList(),
        libraryEntryId = fields?.text("library_entry_id"),
        progressPercent = fields?.get("progress_percent")?.jsonPrimitive?.doubleOrNull?.toInt(),
        triageState = triage ?: "inbox",
        lastSeen = true,
    )
    return document.withProgress(queued.progress(Progress(document.progressPercent, null, null)))
}

private fun JsonObject.text(key: String): String? = runCatching { get(key)?.jsonPrimitive?.contentOrNull }.getOrNull()
