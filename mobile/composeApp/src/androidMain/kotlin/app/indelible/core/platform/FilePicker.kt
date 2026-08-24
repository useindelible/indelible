package app.indelible.core.platform

import android.content.ContentResolver
import android.content.res.AssetFileDescriptor
import android.net.Uri
import android.provider.OpenableColumns
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.runtime.Composable
import androidx.compose.ui.platform.LocalContext
import java.io.ByteArrayOutputStream
import java.io.InputStream

private const val DEFAULT_FILE_NAME = "import.opml"
private const val UNKNOWN_SIZE = -1L
private const val READ_CHUNK_BYTES = 8 * 1024

@Composable
actual fun rememberFilePicker(
    mimeTypes: List<String>,
    maxBytes: Long,
    onFileTooLarge: (name: String) -> Unit,
    onFilePicked: (bytes: ByteArray, name: String) -> Unit,
): () -> Unit {
    val context = LocalContext.current
    val launcher =
        rememberLauncherForActivityResult(
            ActivityResultContracts.OpenDocument(),
        ) { uri: Uri? ->
            if (uri != null) {
                val resolver = context.contentResolver
                val name = uri.lastPathSegment?.substringAfterLast('/') ?: DEFAULT_FILE_NAME
                val declaredSize = resolver.pickedFileSize(uri)
                if (declaredSize > maxBytes) {
                    onFileTooLarge(name)
                } else {
                    // Providers may report no size at all, and a provider that
                    // reports one can still stream more, so the read itself is
                    // bounded: one byte past the cap is enough to reject.
                    val bytes = resolver.openInputStream(uri)?.use { it.readAtMost(maxBytes + 1) }
                    when {
                        bytes == null -> Unit
                        bytes.size > maxBytes -> onFileTooLarge(name)
                        else -> onFilePicked(bytes, name)
                    }
                }
            }
        }
    return { launcher.launch(mimeTypes.toTypedArray()) }
}

/** Returns the picked file's size in bytes, or [UNKNOWN_SIZE] when the provider does not expose one. */
private fun ContentResolver.pickedFileSize(uri: Uri): Long {
    runCatching {
        query(uri, arrayOf(OpenableColumns.SIZE), null, null, null)?.use { cursor ->
            val column = cursor.getColumnIndex(OpenableColumns.SIZE)
            if (column >= 0 && cursor.moveToFirst() && !cursor.isNull(column)) {
                return cursor.getLong(column)
            }
        }
    }
    return runCatching { openAssetFileDescriptor(uri, "r")?.use { it.length } }
        .getOrNull()
        ?.takeIf { it != AssetFileDescriptor.UNKNOWN_LENGTH }
        ?: UNKNOWN_SIZE
}

private fun InputStream.readAtMost(limit: Long): ByteArray {
    val collected = ByteArrayOutputStream()
    val chunk = ByteArray(READ_CHUNK_BYTES)
    var total = 0L
    while (total < limit) {
        val wanted = minOf(chunk.size.toLong(), limit - total).toInt()
        val read = read(chunk, 0, wanted)
        if (read == -1) break
        collected.write(chunk, 0, read)
        total += read
    }
    return collected.toByteArray()
}
