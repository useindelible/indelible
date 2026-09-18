package app.indelible.core.offline

import app.indelible.db.Cached_highlight
import app.indelible.db.OfflineQueries

/**
 * The [EnqueueTx] receiver only makes sense while its backing transaction is open; a caller
 * that stashes it and calls it later would otherwise run queries outside any transaction
 * boundary. [close] is called once [SqlDelightOutbox.enqueue]'s transaction finishes
 * (success or failure), after which every method throws.
 */
internal class EnqueueTxImpl(
    private val scope: String,
    private val queries: OfflineQueries,
) : EnqueueTx {
    private var closed = false

    fun close() {
        closed = true
    }

    private fun checkOpen() {
        check(!closed) { "EnqueueTx cannot be used after its enqueue() transaction has finished" }
    }

    override fun allocateOriginSeq(): Long {
        checkOpen()
        val state = queries.getClientState(scope).executeAsOneOrNull() ?: throw ScopeNotLiveException(scope)
        val current = state.next_origin_seq
        queries.setNextOriginSeq(current + 1, scope)
        return current
    }

    override fun getCachedHighlight(id: String): CachedHighlightRow? {
        checkOpen()
        return queries.cachedHighlight(scope, id).executeAsOneOrNull()?.let(::cachedHighlightRowFrom)
    }

    override fun upsertCachedHighlight(
        id: String,
        documentId: String,
        payloadJson: String,
        updatedAt: Long,
    ) {
        checkOpen()
        queries.upsertCachedHighlight(scope, id, documentId, payloadJson, updatedAt)
    }

    override fun deleteCachedHighlight(id: String) {
        checkOpen()
        queries.deleteCachedHighlight(scope, id)
    }

    override fun setCachedNote(
        documentId: String,
        body: String,
    ) {
        checkOpen()
        queries.setCachedNote(body, scope, documentId)
    }

    override fun patchCachedProgress(
        documentId: String,
        percent: Int,
    ) {
        checkOpen()
        queries.patchCachedProgress(percent = percent.toLong(), scope = scope, document_id = documentId)
    }
}

private fun cachedHighlightRowFrom(row: Cached_highlight): CachedHighlightRow =
    CachedHighlightRow(
        id = row.id,
        documentId = row.document_id,
        payloadJson = row.payload_json,
        updatedAt = row.updated_at,
    )
