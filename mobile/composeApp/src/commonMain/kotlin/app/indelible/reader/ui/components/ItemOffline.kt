package app.indelible.reader.ui.components

import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import app.indelible.offline.viewmodel.DocumentOfflineStatus
import app.indelible.offline.viewmodel.DocumentOfflineViewModel

/** The open document's offline status, the way to Storage, and Keep for a document Storage cannot list. */
class ItemOffline(
    val status: DocumentOfflineStatus,
    val onOpenStorage: () -> Unit = {},
    val onKeep: () -> Unit = {},
)

/** Follows [viewModel]'s document; none without one. */
@Composable
fun rememberItemOffline(
    viewModel: DocumentOfflineViewModel?,
    onOpenStorage: () -> Unit,
): ItemOffline? {
    if (viewModel == null) return null
    val status by viewModel.status.collectAsState()
    return remember(status, viewModel, onOpenStorage) { ItemOffline(status, onOpenStorage, viewModel::keep) }
}
