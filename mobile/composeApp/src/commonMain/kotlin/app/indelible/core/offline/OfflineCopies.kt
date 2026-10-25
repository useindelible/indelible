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

    /** Evicts unpinned copies to the cap, oversize then oldest opened, except copies for which [keep] answers true. */
    suspend fun enforceCap(
        scope: String,
        keep: suspend (String) -> Boolean,
    ): Boolean {
        val cap = capBytes()
        var total = store.totalBytes(scope)
        var evicted = false
        val unpinned = store.unpinnedLru(scope)
        val candidates = (unpinned.filter { it.bytes > cap } + unpinned.filter { it.bytes <= cap }).iterator()
        while (total > cap && candidates.hasNext()) {
            val copy = candidates.next()
            if (keep(copy.documentId)) continue
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
    ): Boolean = store.cachedDocument(scope, documentId)?.let { isIntact(scope, it) } == true

    /** A file that cannot be read counts as missing, so the copy is repaired rather than trusted. */
    internal suspend fun isIntact(
        scope: String,
        copy: CachedDocumentRow,
    ): Boolean {
        val dir = files.generationDir(scope, copy.documentId, copy.generation)
        return try {
            store.assetsForDocument(scope, copy.documentId).all { files.size(dir / it.path) == it.bytes }
        } catch (ignored: IOException) {
            false
        }
    }

    internal fun deleteQuietly(path: Path) {
        try {
            files.deleteTree(path)
        } catch (ignored: IOException) {
        }
    }
}
