package app.indelible.core.platform

import android.net.Uri
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.runtime.Composable
import androidx.compose.ui.platform.LocalContext

@Composable
actual fun rememberFilePicker(
    mimeTypes: List<String>,
    onFilePicked: (bytes: ByteArray, name: String) -> Unit,
): () -> Unit {
    val context = LocalContext.current
    val launcher =
        rememberLauncherForActivityResult(
            ActivityResultContracts.OpenDocument(),
        ) { uri: Uri? ->
            if (uri != null) {
                val bytes = context.contentResolver.openInputStream(uri)?.use { it.readBytes() }
                if (bytes != null) {
                    val name = uri.lastPathSegment?.substringAfterLast('/') ?: "import.opml"
                    onFilePicked(bytes, name)
                }
            }
        }
    return { launcher.launch(mimeTypes.toTypedArray()) }
}
