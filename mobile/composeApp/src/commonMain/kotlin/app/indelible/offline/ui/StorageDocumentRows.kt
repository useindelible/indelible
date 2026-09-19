package app.indelible.offline.ui

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextOverflow
import app.indelible.core.offline.CatalogEntry
import app.indelible.offline.viewmodel.StorageUiState
import app.indelible.profile.ui.components.SettingsSection
import app.indelible.ui.theme.IndelibleSpacing
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.offline_cached
import indelible.composeapp.generated.resources.offline_kept
import indelible.composeapp.generated.resources.offline_manage_kept
import indelible.composeapp.generated.resources.offline_manage_kept_body
import indelible.composeapp.generated.resources.offline_nothing_downloaded
import indelible.composeapp.generated.resources.offline_remove_copy
import org.jetbrains.compose.resources.StringResource
import org.jetbrains.compose.resources.stringResource

internal fun LazyListScope.documentSections(
    state: StorageUiState,
    actions: StorageActions,
) {
    if (state.kept.isEmpty() && state.cached.isEmpty()) {
        item { StorageNote(stringResource(Res.string.offline_nothing_downloaded)) }
        return
    }
    if (state.keptOverCap) {
        item {
            SettingsSection(title = stringResource(Res.string.offline_manage_kept)) {
                StorageNote(stringResource(Res.string.offline_manage_kept_body))
            }
        }
    }
    documentList(Res.string.offline_kept, state.kept, actions)
    documentList(Res.string.offline_cached, state.cached, actions)
}

private fun LazyListScope.documentList(
    title: StringResource,
    entries: List<CatalogEntry>,
    actions: StorageActions,
) {
    if (entries.isEmpty()) return
    item { SettingsSection(title = stringResource(title)) {} }
    items(entries, key = { "document:${it.documentId}" }) { entry ->
        DocumentRow(
            entry = entry,
            onOpen = { actions.onOpenReader(entry.documentId) },
            onRemove = { actions.onRemoveCopy(entry.documentId) },
        )
    }
}

@Composable
private fun DocumentRow(
    entry: CatalogEntry,
    onOpen: () -> Unit,
    onRemove: () -> Unit,
) {
    Row(
        modifier =
            Modifier
                .fillMaxWidth()
                .clickable(onClick = onOpen)
                .padding(start = IndelibleSpacing.rowPaddingH, end = IndelibleSpacing.step8),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Column(
            modifier =
                Modifier
                    .weight(1f)
                    .padding(vertical = IndelibleSpacing.step8),
        ) {
            Text(
                text = entry.title,
                style = MaterialTheme.typography.titleSmall,
                maxLines = 2,
                overflow = TextOverflow.Ellipsis,
            )
            Text(
                text = byteSize(entry.bytes),
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
        IconButton(onClick = onRemove) {
            Icon(
                imageVector = Icons.Filled.Delete,
                contentDescription = stringResource(Res.string.offline_remove_copy),
                tint = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
    }
}
