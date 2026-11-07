package app.indelible.core.offline

import okio.FileSystem
import okio.Path.Companion.toPath
import java.io.File

actual class OfflineFilesRoot(
    private val directory: File = File(System.getProperty("user.home"), ".indelible"),
) {
    actual fun offlineFiles(): OfflineFiles =
        OfflineFiles(directory.absolutePath.toPath() / OFFLINE_DIRECTORY, FileSystem.SYSTEM)

    private companion object {
        const val OFFLINE_DIRECTORY = "offline"
    }
}
