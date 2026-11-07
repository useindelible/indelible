package app.indelible.core.offline

/** Must run while no acquisition does; every generation on disk is then either referenced or garbage. */
internal suspend fun OfflineCopies.sweep() {
    val scopes = store.scopesWithState()
    val known = scopes.map { (scope, _) -> scopeDirName(scope) }.toSet()
    files.listDirectories(files.root).filter { it.name !in known }.forEach(::deleteQuietly)
    scopes.filterNot { (_, purgePending) -> purgePending }.forEach { (scope, _) -> sweepScope(scope) }
}

private suspend fun OfflineCopies.sweepScope(scope: String) {
    val (intact, broken) = store.cachedDocuments(scope).partition { isIntact(scope, it) }
    // A kept copy with a missing file is repaired on its next open; dropping it here would lose the pin.
    val (kept, garbage) = broken.partition { it.pinned }
    garbage.forEach { remove(scope, it.documentId) }
    val generations = (intact + kept).associate { it.documentId to it.generation.toString() }
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
