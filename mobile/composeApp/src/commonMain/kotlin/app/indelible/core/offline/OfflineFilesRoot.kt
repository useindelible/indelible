package app.indelible.core.offline

/** Where this platform keeps downloaded documents. */
expect class OfflineFilesRoot {
    fun offlineFiles(): OfflineFiles
}
