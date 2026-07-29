package app.indelible.core.platform

import androidx.compose.runtime.Composable

@Composable
expect fun rememberFilePicker(
    mimeTypes: List<String>,
    onFilePicked: (bytes: ByteArray, name: String) -> Unit,
): () -> Unit
