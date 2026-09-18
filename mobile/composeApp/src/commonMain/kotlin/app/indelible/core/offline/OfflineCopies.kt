package app.indelible.core.offline

import okio.IOException
import okio.Path

/**
 * The offline copies on this device. A copy is removed rows first, then files: once its rows are
 * gone nothing reads the files, so a delete that fails leaves only a directory the launch sweep
 * reclaims.
 */
class OfflineCopies(
    internal val store: OfflineStore,
    internal val files: OfflineFiles,
    private val capBytes: suspend () -> Long,
) {
    suspend fun remove(
        scope: String,
        documentId: String,
    ) {
        store.removeCachedDocument(scope, documentId)
        deleteQuietly(files.documentDir(scope, documentId))
    }

    /** Removes every copy of [scope]; a failed delete is thrown, so a purge stays pending and is retried. */
    suspend fun removeAll(scope: String) {
        store.cachedDocuments(scope).forEach { store.removeCachedDocument(scope, it.documentId) }
        files.deleteTree(files.scopeDir(scope))
    }

    /**
     * Evicts unpinned copies of [scope], least recently opened first, until it fits under the cap.
     * Pinned copies are never evicted, so kept items alone can leave the scope over the cap.
     */
    suspend fun enforceCap(
        scope: String,
        keep: String?,
    ) {
        val cap = capBytes()
        var total = store.totalBytes(scope)
        val candidates = store.unpinnedLru(scope).filter { it.documentId != keep }.iterator()
        while (total > cap && candidates.hasNext()) {
            val copy = candidates.next()
            remove(scope, copy.documentId)
            total -= copy.bytes
        }
    }

    internal fun deleteQuietly(path: Path) {
        try {
            files.deleteTree(path)
        } catch (ignored: IOException) {
            // The launch sweep deletes every directory no row references.
        }
    }
}
