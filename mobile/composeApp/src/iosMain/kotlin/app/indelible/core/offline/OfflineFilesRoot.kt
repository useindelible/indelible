package app.indelible.core.offline

import kotlinx.cinterop.ExperimentalForeignApi
import okio.FileSystem
import okio.Path.Companion.toPath
import platform.Foundation.NSApplicationSupportDirectory
import platform.Foundation.NSFileManager
import platform.Foundation.NSURLIsExcludedFromBackupKey
import platform.Foundation.NSUserDomainMask

actual class OfflineFilesRoot {
    @OptIn(ExperimentalForeignApi::class)
    actual fun offlineFiles(): OfflineFiles {
        val manager = NSFileManager.defaultManager
        val support =
            checkNotNull(manager.URLForDirectory(NSApplicationSupportDirectory, NSUserDomainMask, null, true, null)) {
                "Application Support is unavailable"
            }
        val offline = checkNotNull(support.URLByAppendingPathComponent(OFFLINE_DIRECTORY, isDirectory = true))
        manager.createDirectoryAtURL(offline, withIntermediateDirectories = true, attributes = null, error = null)
        // Downloads can always be fetched again, so they stay out of device and iCloud backups.
        offline.setResourceValue(true, forKey = NSURLIsExcludedFromBackupKey, error = null)
        return OfflineFiles(checkNotNull(offline.path).toPath(), FileSystem.SYSTEM)
    }

    private companion object {
        const val OFFLINE_DIRECTORY = "offline"
    }
}
