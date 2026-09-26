package app.indelible.core.offline

import io.ktor.utils.io.ByteReadChannel

/** Installs a copy of [documentId] with one readable file of [bytes] bytes, opened at [openedAt]. */
internal suspend fun DownloadHarness.seed(
    documentId: String,
    bytes: Long,
    openedAt: Long = 0L,
    pinned: Boolean = false,
) {
    val asset = CachedAssetRow(documentId, "readable_html", 0, "readable.html", bytes)
    val dir = files.generationDir(DL_SCOPE, documentId, 1)
    files.write(dir / asset.path, ByteReadChannel(ByteArray(bytes.toInt())))
    val request =
        InstallRequest(
            documentId = documentId,
            documentType = "article",
            revision = store.localChanges(DL_SCOPE, documentId).revision,
            server = serverDocument(),
            generation = 1,
            assets = listOf(asset),
            bytes = bytes,
            pin = pinned,
            at = openedAt,
        )
    check(store.installCachedDocument(session, request) == InstallResult.Installed)
}

internal suspend fun DownloadHarness.copyIds(): Set<String> =
    store
        .cachedDocuments(DL_SCOPE)
        .map { it.documentId }
        .toSet()

/** A catalog row written without files or a session, the way a retained scope's rows sit. */
internal fun catalogRow(
    documentId: String,
    bytes: Long,
) = CachedDocumentRow(
    documentId = documentId,
    documentType = "article",
    title = "Title",
    readerJson = "{}",
    pinned = false,
    lastOpenedAt = 1L,
    lastSyncedAt = null,
    bytes = bytes,
)
