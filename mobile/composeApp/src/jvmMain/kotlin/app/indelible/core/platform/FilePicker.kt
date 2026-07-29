package app.indelible.core.platform

import androidx.compose.runtime.Composable

@Composable
actual fun rememberFilePicker(
    mimeTypes: List<String>,
    onFilePicked: (bytes: ByteArray, name: String) -> Unit,
): () -> Unit = {}
