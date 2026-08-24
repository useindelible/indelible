package app.indelible.core.platform

import androidx.compose.runtime.Composable

/**
 * Picks a single file and hands its bytes to [onFilePicked].
 *
 * Every implementation determines the file's size before reading it, and where a
 * platform cannot report a size the read itself is bounded, so a pick never puts
 * more than [maxBytes] + 1 bytes in memory. Anything larger than [maxBytes] is
 * reported through [onFileTooLarge] and never handed to [onFilePicked].
 */
@Composable
expect fun rememberFilePicker(
    mimeTypes: List<String>,
    maxBytes: Long,
    onFileTooLarge: (name: String) -> Unit,
    onFilePicked: (bytes: ByteArray, name: String) -> Unit,
): () -> Unit
