package app.indelible.core.offline

import io.ktor.utils.io.ByteReadChannel
import io.ktor.utils.io.readAvailable
import okio.BufferedSink
import okio.FileSystem
import okio.IOException
import okio.Path
import okio.buffer
import okio.use

class NoSpaceException(
    cause: Throwable,
) : IOException("No space left on device", cause)

/**
 * Downloaded document files under one root. Writes stream straight into their final path: each
 * download owns a fresh generation directory, so there is nothing to stage or move.
 */
class OfflineFiles(
    val root: Path,
    private val fs: FileSystem,
) {
    /** Streams [channel] into [path], creating its parents, and returns the bytes written. */
    suspend fun write(
        path: Path,
        channel: ByteReadChannel,
    ): Long =
        try {
            path.parent?.let(fs::createDirectories)
            fs.sink(path).buffer().use { sink -> copy(channel, sink) }
        } catch (error: IOException) {
            throw if (error.isNoSpace()) NoSpaceException(error) else error
        }

    private suspend fun copy(
        channel: ByteReadChannel,
        sink: BufferedSink,
    ): Long {
        val buffer = ByteArray(COPY_BUFFER_BYTES)
        var total = 0L
        while (true) {
            val read = channel.readAvailable(buffer, 0, buffer.size)
            if (read == -1) return total
            sink.write(buffer, 0, read)
            total += read
        }
    }

    fun readText(path: Path): String? = if (fs.exists(path)) fs.read(path) { readUtf8() } else null

    fun size(path: Path): Long? = fs.metadataOrNull(path)?.size

    fun exists(path: Path): Boolean = fs.exists(path)

    fun deleteTree(path: Path) {
        fs.deleteRecursively(path, mustExist = false)
    }

    fun listDirectories(path: Path): List<Path> =
        fs.listOrNull(path).orEmpty().filter { fs.metadataOrNull(it)?.isDirectory == true }

    fun sizeOfTree(path: Path): Long {
        if (!fs.exists(path)) return 0L
        return fs
            .listRecursively(path)
            .mapNotNull { fs.metadataOrNull(it) }
            .filter { it.isRegularFile }
            .sumOf { it.size ?: 0L }
    }

    private companion object {
        const val COPY_BUFFER_BYTES = 64 * 1024
    }
}

// Android reports ENOSPC through ErrnoException text, the JVM and Okio's native file system
// through strerror, and either may be wrapped by the stream that hit it.
internal fun Throwable.isNoSpace(): Boolean =
    generateSequence(this) { it.cause }.any { error ->
        val message = error.message.orEmpty()
        message.contains("ENOSPC") || message.contains("No space left on device", ignoreCase = true)
    }
