package app.indelible.library.repository

private const val WHEN = "2026-01-01T00:00:00Z"

internal fun libraryEntryJson(
    id: String,
    documentId: String = "doc_${id.removePrefix("entry_")}",
    type: String = "pdf",
    triage: String = "inbox",
    progress: Int = 40,
): String =
    """{"library_entry_id":"$id","document_id":"$documentId","object":"library_entry",""" +
        """"progress_percent":$progress,"max_progress_percent":$progress,"last_read_at":null,"finished_at":null,""" +
        """"title":"Title $id","source":"web","document_type":"$type","triage_state":"$triage",""" +
        """"is_favorite":false,"is_shortlisted":false,"saved_at":"$WHEN","created_at":"$WHEN","updated_at":"$WHEN"}"""
