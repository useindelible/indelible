package app.indelible.core.platform

import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import kotlinx.cinterop.BetaInteropApi
import kotlinx.cinterop.ExperimentalForeignApi
import kotlinx.cinterop.addressOf
import kotlinx.cinterop.usePinned
import platform.Foundation.NSData
import platform.Foundation.NSFileManager
import platform.Foundation.NSFileSize
import platform.Foundation.NSNumber
import platform.Foundation.NSURL
import platform.Foundation.dataWithContentsOfURL
import platform.UIKit.UIApplication
import platform.UIKit.UIDocumentPickerDelegateProtocol
import platform.UIKit.UIDocumentPickerViewController
import platform.UniformTypeIdentifiers.UTTypeXML
import platform.darwin.NSObject
import platform.posix.memcpy

private const val DEFAULT_FILE_NAME = "import.opml"

@OptIn(ExperimentalForeignApi::class)
@Composable
actual fun rememberFilePicker(
    mimeTypes: List<String>,
    maxBytes: Long,
    onFileTooLarge: (name: String) -> Unit,
    onFilePicked: (bytes: ByteArray, name: String) -> Unit,
): () -> Unit {
    val delegate =
        remember(maxBytes, onFileTooLarge, onFilePicked) {
            DocumentPickerDelegate(maxBytes, onFileTooLarge, onFilePicked)
        }
    return {
        val picker =
            UIDocumentPickerViewController(
                forOpeningContentTypes = listOf(UTTypeXML),
                asCopy = true,
            )
        picker.delegate = delegate
        picker.allowsMultipleSelection = false
        val rootVc = UIApplication.sharedApplication.keyWindow?.rootViewController
        rootVc?.presentViewController(picker, animated = true, completion = null)
    }
}

@OptIn(ExperimentalForeignApi::class, BetaInteropApi::class)
private class DocumentPickerDelegate(
    private val maxBytes: Long,
    private val onFileTooLarge: (String) -> Unit,
    private val onFilePicked: (ByteArray, String) -> Unit,
) : NSObject(),
    UIDocumentPickerDelegateProtocol {
    override fun documentPicker(
        controller: UIDocumentPickerViewController,
        didPickDocumentsAtURLs: List<*>,
    ) {
        val url = didPickDocumentsAtURLs.firstOrNull() as? NSURL ?: return
        val name = url.lastPathComponent ?: DEFAULT_FILE_NAME
        // The picker copies the file into the app's container, so its size is
        // always readable here; a size we cannot read counts as over the limit
        // rather than loading a file of unknown length into memory.
        val size = url.fileSizeOrNull()
        val withinCap = size != null && size <= maxBytes
        val data = if (withinCap) NSData.dataWithContentsOfURL(url) else null
        when {
            !withinCap -> onFileTooLarge(name)
            data == null -> Unit
            data.length.toLong() > maxBytes -> onFileTooLarge(name)
            else -> onFilePicked(data.toByteArray(), name)
        }
    }

    override fun documentPickerWasCancelled(controller: UIDocumentPickerViewController) {}
}

@OptIn(ExperimentalForeignApi::class)
private fun NSURL.fileSizeOrNull(): Long? {
    val attributes = path?.let { NSFileManager.defaultManager.attributesOfItemAtPath(it, null) }
    return (attributes?.get(NSFileSize) as? NSNumber)?.longLongValue
}

@OptIn(ExperimentalForeignApi::class)
private fun NSData.toByteArray(): ByteArray {
    val copy = ByteArray(length.toInt())
    if (copy.isNotEmpty()) {
        copy.usePinned { pinned -> memcpy(pinned.addressOf(0), bytes, length) }
    }
    return copy
}
