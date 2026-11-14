package app.indelible.reader.ui.components

import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import app.indelible.offline.viewmodel.DocumentOfflineStatus
import app.indelible.offline.viewmodel.DocumentOfflineViewModel

/** The open document's offline status, and the way from the reader to Storage. */
class ItemOffline(
    val status: DocumentOfflineStatus,
    val onOpenStorage: () -> Unit = {},
)

/** Follows [viewModel]'s document; none without one. */
@Composable
fun rememberItemOffline(
    viewModel: DocumentOfflineViewModel?,
    onOpenStorage: () -> Unit,
): ItemOffline? {
    if (viewModel == null) return null
    val status by viewModel.status.collectAsState()
    return remember(status, onOpenStorage) { ItemOffline(status, onOpenStorage) }
}
