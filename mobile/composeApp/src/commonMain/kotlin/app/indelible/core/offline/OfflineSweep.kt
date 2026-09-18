package app.indelible.core.offline

/**
 * Reconciles the files with the catalog after a crash, a failed delete or an interrupted download.
 * It must run while no acquisition does: every generation on disk is then either the one its row
 * references or garbage. A purge-pending scope is left to its purge, which owns its files.
 */
internal suspend fun OfflineCopies.sweep() {
    val scopes = store.scopesWithState()
    val known = scopes.map { (scope, _) -> scopeDirName(scope) }.toSet()
    files.listDirectories(files.root).filter { it.name !in known }.forEach(::deleteQuietly)
    scopes.filterNot { (_, purgePending) -> purgePending }.forEach { (scope, _) -> sweepScope(scope) }
}

private suspend fun OfflineCopies.sweepScope(scope: String) {
    val (intact, broken) = store.cachedDocuments(scope).partition { isIntact(scope, it) }
    broken.forEach { remove(scope, it.documentId) }
    val generations = intact.associate { it.documentId to it.generation.toString() }
    for (documentDir in files.listDirectories(files.scopeDir(scope))) {
        val generation = generations[documentDir.name]
        val unreferenced =
            if (generation == null) {
                listOf(documentDir)
            } else {
                files.listDirectories(documentDir).filter { it.name != generation }
            }
        unreferenced.forEach(::deleteQuietly)
    }
    store.dropOrphanHighlights(scope)
}

private suspend fun OfflineCopies.isIntact(
    scope: String,
    copy: CachedDocumentRow,
): Boolean {
    val dir = files.generationDir(scope, copy.documentId, copy.generation)
    return store.assetsForDocument(scope, copy.documentId).all { files.size(dir / it.path) == it.bytes }
}
