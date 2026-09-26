package app.indelible.core.offline

import okio.IOException
import okio.Path

/** Removes rows before files, so a failed file delete leaves only a directory the sweep reclaims. */
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

    /** Evicts unpinned copies, least recently opened first, down to the cap; true if it freed any. */
    suspend fun enforceCap(
        scope: String,
        keep: String?,
    ): Boolean {
        val cap = capBytes()
        var total = store.totalBytes(scope)
        var evicted = false
        val candidates = store.unpinnedLru(scope).filter { it.documentId != keep }.iterator()
        while (total > cap && candidates.hasNext()) {
            val copy = candidates.next()
            if (store.evictCachedDocument(scope, copy.documentId)) {
                deleteQuietly(files.documentDir(scope, copy.documentId))
                total -= copy.bytes
                evicted = true
            }
        }
        return evicted
    }

    internal fun deleteQuietly(path: Path) {
        try {
            files.deleteTree(path)
        } catch (ignored: IOException) {
        }
    }
}
