package app.indelible.core.platform

import androidx.compose.runtime.Composable

// Desktop has no file picker yet; the size cap lives with the implementation,
// so a future actual must check File.length() against maxBytes before reading.
@Composable
actual fun rememberFilePicker(
    mimeTypes: List<String>,
    maxBytes: Long,
    onFileTooLarge: (name: String) -> Unit,
    onFilePicked: (bytes: ByteArray, name: String) -> Unit,
): () -> Unit = {}
