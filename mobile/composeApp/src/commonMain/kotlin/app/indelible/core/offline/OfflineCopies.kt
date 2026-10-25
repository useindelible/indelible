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

    /** Evicts unpinned copies not in [keep] down to the cap, oversize ones first, then least recently opened. */
    suspend fun enforceCap(
        scope: String,
        keep: Set<String>,
    ): Boolean {
        val cap = capBytes()
        var total = store.totalBytes(scope)
        var evicted = false
        val unpinned = store.unpinnedLru(scope).filter { it.documentId !in keep }
        val candidates = (unpinned.filter { it.bytes > cap } + unpinned.filter { it.bytes <= cap }).iterator()
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

    /** True when every file the copy's catalog lists is present at its recorded size. */
    suspend fun isComplete(
        scope: String,
        documentId: String,
    ): Boolean {
        val copy = store.cachedDocument(scope, documentId) ?: return false
        val dir = files.generationDir(scope, documentId, copy.generation)
        return store.assetsForDocument(scope, documentId).all { files.size(dir / it.path) == it.bytes }
    }

    internal fun deleteQuietly(path: Path) {
        try {
            files.deleteTree(path)
        } catch (ignored: IOException) {
        }
    }
}
