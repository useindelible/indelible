package app.indelible.offline.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.TextUnit
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.em
import app.indelible.core.offline.CatalogEntry
import app.indelible.offline.viewmodel.StorageUiState
import app.indelible.ui.theme.IndelibleSpacing
import app.indelible.ui.theme.IndelibleTheme
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.offline_downloads
import indelible.composeapp.generated.resources.offline_filter_all
import indelible.composeapp.generated.resources.offline_filter_cached
import indelible.composeapp.generated.resources.offline_filter_kept
import indelible.composeapp.generated.resources.offline_inventory_count
import indelible.composeapp.generated.resources.offline_items
import indelible.composeapp.generated.resources.offline_note_all
import indelible.composeapp.generated.resources.offline_note_cached
import indelible.composeapp.generated.resources.offline_note_kept
import indelible.composeapp.generated.resources.offline_nothing_downloaded
import indelible.composeapp.generated.resources.offline_remove
import indelible.composeapp.generated.resources.offline_unpin
import indelible.composeapp.generated.resources.offline_view_empty
import indelible.composeapp.generated.resources.reader_type_article
import indelible.composeapp.generated.resources.reader_type_book
import indelible.composeapp.generated.resources.reader_type_pdf
import indelible.composeapp.generated.resources.reader_type_unknown
import indelible.composeapp.generated.resources.reader_type_video
import org.jetbrains.compose.resources.StringResource
import org.jetbrains.compose.resources.pluralStringResource
import org.jetbrains.compose.resources.stringResource

private val MARK_DOT = 5.dp
private val TRACK_MARK = 0.11.em

enum class DownloadFilter { ALL, KEPT, CACHED }

internal fun LazyListScope.documentSections(
    state: StorageUiState,
    filter: DownloadFilter,
    onFilterChange: (DownloadFilter) -> Unit,
    actions: StorageActions,
) {
    if (state.kept.isEmpty() && state.cached.isEmpty()) {
        item { StorageNote(stringResource(Res.string.offline_nothing_downloaded)) }
        return
    }
    item { DownloadsHeader(state, filter, onFilterChange) }
    val entries = state.inView(filter).sortedByDescending { it.bytes }
    if (entries.isEmpty()) {
        item { StorageNote(stringResource(Res.string.offline_view_empty)) }
    }
    itemsIndexed(entries, key = { _, entry -> "document:${entry.documentId}" }) { index, entry ->
        DocumentRow(
            entry = entry,
            showMark = filter == DownloadFilter.ALL,
            first = index == 0,
            last = index == entries.lastIndex,
            onOpen = { actions.onOpenReader(entry.documentId) },
            onAct = { if (entry.pinned) actions.onUnpin(entry.documentId) else actions.onRemoveCopy(entry.documentId) },
        )
    }
    item { StorageNote(stringResource(noteFor(filter))) }
}

private fun StorageUiState.inView(filter: DownloadFilter): List<CatalogEntry> =
    when (filter) {
        DownloadFilter.ALL -> kept + cached
        DownloadFilter.KEPT -> kept
        DownloadFilter.CACHED -> cached
    }

@Composable
private fun DownloadsHeader(
    state: StorageUiState,
    filter: DownloadFilter,
    onFilterChange: (DownloadFilter) -> Unit,
) {
    val inView = state.inView(filter)
    Column(
        modifier =
            Modifier
                .fillMaxWidth()
                .padding(
                    start = IndelibleSpacing.rowPaddingH,
                    end = IndelibleSpacing.rowPaddingH,
                    top = SECTION_SPACING,
                    bottom = IndelibleSpacing.step12,
                ),
    ) {
        SectionHead(
            label = stringResource(Res.string.offline_downloads),
            count =
                stringResource(
                    Res.string.offline_inventory_count,
                    pluralStringResource(Res.plurals.offline_items, inView.size, inView.size),
                    byteSize(inView.sumOf { it.bytes }),
                ),
        )
        StorageSegmented(
            segments = DownloadFilter.entries.map { Segment(stringResource(filterLabel(it)), state.inView(it).size) },
            selected = filter.ordinal,
            onSelect = { onFilterChange(DownloadFilter.entries[it]) },
            modifier = Modifier.padding(top = IndelibleSpacing.step12),
        )
    }
}

private fun filterLabel(filter: DownloadFilter): StringResource =
    when (filter) {
        DownloadFilter.ALL -> Res.string.offline_filter_all
        DownloadFilter.KEPT -> Res.string.offline_filter_kept
        DownloadFilter.CACHED -> Res.string.offline_filter_cached
    }

private fun noteFor(filter: DownloadFilter): StringResource =
    when (filter) {
        DownloadFilter.ALL -> Res.string.offline_note_all
        DownloadFilter.KEPT -> Res.string.offline_note_kept
        DownloadFilter.CACHED -> Res.string.offline_note_cached
    }

private fun typeLabel(documentType: String): StringResource =
    when (documentType.lowercase()) {
        "article" -> Res.string.reader_type_article
        "video" -> Res.string.reader_type_video
        "pdf" -> Res.string.reader_type_pdf
        "book", "epub" -> Res.string.reader_type_book
        else -> Res.string.reader_type_unknown
    }

@Composable
private fun DocumentRow(
    entry: CatalogEntry,
    showMark: Boolean,
    first: Boolean,
    last: Boolean,
    onOpen: () -> Unit,
    onAct: () -> Unit,
) {
    val colors = MaterialTheme.colorScheme
    Row(
        modifier =
            Modifier
                .padding(horizontal = IndelibleSpacing.rowPaddingH)
                .fillMaxWidth()
                .cardCell(first, last, colors.surfaceVariant, colors.outlineVariant)
                .clickable(onClick = onOpen)
                .padding(horizontal = IndelibleSpacing.step14, vertical = IndelibleSpacing.step12),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step12),
    ) {
        Column(modifier = Modifier.weight(1f)) {
            Text(
                text = entry.title,
                style = MaterialTheme.typography.titleSmall,
                color = colors.onSurface,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
            RowMeta(entry, showMark)
        }
        Text(
            text = byteSize(entry.bytes),
            style = storageMono(TextUnit.Unspecified),
            color = colors.onSurfaceVariant,
        )
        RowAction(
            text = stringResource(if (entry.pinned) Res.string.offline_unpin else Res.string.offline_remove),
            onClick = onAct,
        )
    }
}

/** The durability mark, then the type, in the mono caption the prototype gives row metadata. */
@Composable
private fun RowMeta(
    entry: CatalogEntry,
    showMark: Boolean,
) {
    val quiet = IndelibleTheme.colors.textTertiary
    Row(
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step6),
        modifier = Modifier.padding(top = IndelibleSpacing.step4),
    ) {
        if (showMark) {
            val ink = if (entry.pinned) IndelibleTheme.colors.success else quiet
            Box(
                modifier =
                    Modifier
                        .size(MARK_DOT)
                        .clip(CircleShape)
                        .background(ink),
            )
            Text(
                text = stringResource(filterLabel(if (entry.pinned) DownloadFilter.KEPT else DownloadFilter.CACHED)),
                style = storageMono(TRACK_MARK),
                color = ink,
            )
        }
        Text(
            text = stringResource(typeLabel(entry.documentType)),
            style = storageMono(TRACK_META, FontWeight.Normal),
            color = quiet,
        )
    }
}
