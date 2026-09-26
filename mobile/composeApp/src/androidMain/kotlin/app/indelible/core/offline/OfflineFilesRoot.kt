package app.indelible.core.offline

import android.content.Context
import okio.FileSystem
import okio.Path.Companion.toPath

actual class OfflineFilesRoot(
    private val context: Context,
) {
    actual fun offlineFiles(): OfflineFiles =
        OfflineFiles(context.filesDir.absolutePath.toPath() / OFFLINE_DIRECTORY, FileSystem.SYSTEM)

    private companion object {
        const val OFFLINE_DIRECTORY = "offline"
    }
}
